use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Debug;

use mockall::predicate;

use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::Interpreter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::player::MockPatternPlayer;
use crate::player::unit::SoundUnit;
use crate::structures::index::Index;
use crate::test::interpreter::logging::LoggingPlayer;

mod arbitrary;
mod logging;
mod sequence;
mod unittests;

/// Minimum threshold for which a played element's start time is considered
/// equal to the expected start time.
const EPSILON: CycleTime = CycleTime::from_int_recip(4096);

/// A triple of an expected scheduled start time, duration, and note unit.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ScheduledExpectation {
    pub start_time: CycleTime,
    pub duration: CycleTime,
    pub note_unit: NoteUnit,
}

/// Performs a simulation on a pattern stored in `arenas` with given head index
/// `head_index`. For each element in `expected_actions`, the first denotes
/// the next time to update, and the second denotes a list of expected scheduled
/// events: the scheduled time, duration, and unit. The expectation will be
/// added and checked sequentially for each pair in `expected_actions`.
fn test_expectations<
    'a,
    'b,
    ExpectationsAtTime: IntoIterator<Item = impl Borrow<ScheduledExpectation>> + Clone + 'a + 'b,
    Expectations: IntoIterator<Item = &'b (CycleTime, ExpectationsAtTime)> + Clone,
>(
    arenas: &'a impl PatternArenas,
    head_index: Index<Pattern>,
    expected_schedule_actions: Expectations,
) {
    test_expectations_with_interpreter_setup(
        arenas,
        head_index,
        expected_schedule_actions,
        EmptyInterpreterSetup,
    )
}

pub trait TestSetupStrategy {
    fn setup_interpreter<Arenas, Player, BorrowAdapter>(
        self,
        interpreter: &mut Interpreter<Arenas, Player, BorrowAdapter>,
    );
}

pub struct FullInterpreterSetup {
    pub offset: CycleTime,
    pub multiplier: CycleTime,
}
impl TestSetupStrategy for FullInterpreterSetup {
    fn setup_interpreter<Arenas, Player, BorrowAdapter>(
        self,
        interpreter: &mut Interpreter<Arenas, Player, BorrowAdapter>,
    ) {
        interpreter.set_multiplier(self.multiplier);
        interpreter.set_offset(self.offset);
    }
}

struct EmptyInterpreterSetup;
impl TestSetupStrategy for EmptyInterpreterSetup {
    fn setup_interpreter<Arenas, Player, BorrowAdapter>(
        self,
        _: &mut Interpreter<Arenas, Player, BorrowAdapter>,
    ) {
        // Does nothing.
    }
}

fn test_expectations_with_interpreter_setup<
    ExpectationsAtTime: IntoIterator<Item = impl Borrow<ScheduledExpectation>> + Clone,
    Expectations: IntoIterator<Item = impl Borrow<(CycleTime, ExpectationsAtTime)>> + Clone,
>(
    arenas: &impl PatternArenas,
    head_index: Index<Pattern>,
    expected_schedule_actions: Expectations,
    test_setup: impl TestSetupStrategy,
) {
    test_expectations_with_interpreter_setup_and_start_time(
        arenas,
        head_index,
        CycleTime::ZERO,
        expected_schedule_actions,
        test_setup,
    )
}

fn test_expectations_with_interpreter_setup_and_start_time<
    ExpectationsAtTime: IntoIterator<Item = impl Borrow<ScheduledExpectation>> + Clone,
    Expectations: IntoIterator<Item = impl Borrow<(CycleTime, ExpectationsAtTime)>> + Clone,
>(
    arenas: &impl PatternArenas,
    head_index: Index<Pattern>,
    start_time: CycleTime,
    expected_schedule_actions: Expectations,
    test_setup: impl TestSetupStrategy,
) {
    {
        let mut mock_player = MockPatternPlayer::new();
        mock_player
            .expect_schedule_note_unit()
            .times(..)
            .with(predicate::always(), predicate::always())
            .return_const(());
        let mut logging_player = LoggingPlayer::new(&mut mock_player);
        logging_player.set_logging(true);
        let last_time = expected_schedule_actions
            .clone()
            .into_iter()
            .map(|x| x.borrow().0)
            .last()
            .unwrap();
        let mut interpreter =
            Interpreter::new(head_index.clone(), arenas, &mut logging_player);
        interpreter.update_time(last_time);
    }

    let mut mock_player = MockPatternPlayer::new();
    let logging_player = RefCell::new(LoggingPlayer::new(&mut mock_player));
    let with_mock_player = |f: &dyn Fn(&mut MockPatternPlayer)| {
        let mut borrowed_logger = logging_player.borrow_mut();
        let borrowed_player = borrowed_logger.get_mut_player();
        f(borrowed_player)
    };

    let mut interpreter =
        Interpreter::new_with_refcell(head_index, arenas, &logging_player);
    test_setup.setup_interpreter(&mut interpreter);

    // Advance the interpreter to the start position.
    // First, ignore all possible played notes before the start position.
    logging_player
        .borrow_mut()
        .set_inner_enabled(false);
    interpreter.update_time(start_time);
    logging_player
        .borrow_mut()
        .set_inner_enabled(true);

    // Enable printing log messages, if desired.
    logging_player
        .borrow_mut()
        .set_logging(true);
    let expect_note_unit = |expectation: ScheduledExpectation| {
        let ScheduledExpectation { start_time, duration, note_unit } =
            expectation;
        let sound_unit = SoundUnit::new(note_unit, duration);
        let add_expectation = |borrowed_player: &mut MockPatternPlayer| {
            let abs_diff = |x: CycleTime, y: CycleTime| {
                if x <= y { y - x } else { x - y }
            };
            let approx_eq_start_time =
                predicate::function(move |time: &CycleTime| {
                    abs_diff(start_time, *time) <= EPSILON
                });
            borrowed_player
                .expect_schedule_note_unit()
                .with(predicate::eq(sound_unit.clone()), approx_eq_start_time)
                .once()
                .return_const(());
        };
        with_mock_player(&add_expectation)
    };

    for borrow_scheduled_actions in expected_schedule_actions {
        let (next_cycle_time, expectations) = borrow_scheduled_actions.borrow();
        expectations
            .clone()
            .into_iter()
            .for_each(|e| expect_note_unit(e.borrow().clone()));
        interpreter.update_time(*next_cycle_time);
        with_mock_player(&|player| player.checkpoint());
    }
}
