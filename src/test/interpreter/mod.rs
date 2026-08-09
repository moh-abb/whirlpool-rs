use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Debug;

use mockall::predicate;

use crate::ast::CycleTime;
use crate::ast::NoteUnit;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::Interpreter;
use crate::interpreter::pattern::PatternInterpreter;
use crate::mem::Arena;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::synth::scheduler::MockUnitScheduler;
use crate::synth::unit::SoundUnit;
use crate::test::interpreter::logging::LoggingScheduler;

mod arbitrary;
mod logging;
mod sequence;
mod unittests;

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
    head_index: Index<PatternNode>,
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
    fn setup_interpreter<
        Arenas,
        Scheduler,
        FrameArena,
        SchedulerBorrow,
        FrameArenaBorrow,
    >(
        self,
        interpreter: &mut PatternInterpreter<
            Arenas,
            Scheduler,
            FrameArena,
            SchedulerBorrow,
            FrameArenaBorrow,
        >,
    );
}

pub struct FullInterpreterSetup {
    pub offset: CycleTime,
    pub multiplier: CycleTime,
}
impl TestSetupStrategy for FullInterpreterSetup {
    fn setup_interpreter<
        Arenas,
        Scheduler,
        FrameArena,
        SchedulerBorrow,
        FrameArenaBorrow,
    >(
        self,
        interpreter: &mut PatternInterpreter<
            Arenas,
            Scheduler,
            FrameArena,
            SchedulerBorrow,
            FrameArenaBorrow,
        >,
    ) {
        interpreter.set_multiplier(self.multiplier);
        interpreter.set_offset(self.offset);
    }
}

struct EmptyInterpreterSetup;
impl TestSetupStrategy for EmptyInterpreterSetup {
    fn setup_interpreter<
        Arenas,
        Scheduler,
        FrameArena,
        SchedulerBorrow,
        FrameArenaBorrow,
    >(
        self,
        _: &mut PatternInterpreter<
            Arenas,
            Scheduler,
            FrameArena,
            SchedulerBorrow,
            FrameArenaBorrow,
        >,
    ) {
        // Does nothing.
    }
}

fn test_expectations_with_interpreter_setup<
    ExpectationsAtTime: IntoIterator<Item = impl Borrow<ScheduledExpectation>> + Clone,
    Expectations: IntoIterator<Item = impl Borrow<(CycleTime, ExpectationsAtTime)>> + Clone,
>(
    arenas: &impl PatternArenas,
    head_index: Index<PatternNode>,
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
    head_index: Index<PatternNode>,
    start_time: CycleTime,
    expected_schedule_actions: Expectations,
    test_setup: impl TestSetupStrategy,
) {
    let mut mock_scheduler = MockUnitScheduler::new();
    let logging_scheduler =
        RefCell::new(LoggingScheduler::new(&mut mock_scheduler));
    let with_mock_scheduler = |f: &dyn Fn(&mut MockUnitScheduler)| {
        let mut borrowed_logger = logging_scheduler.borrow_mut();
        let borrowed_scheduler = borrowed_logger.get_mut_scheduler();
        f(borrowed_scheduler)
    };

    let mut frame_arena = GrowableArena::<()>::new();
    let frame_arena_refcell = RefCell::new(&mut frame_arena);
    debug_assert_eq!(frame_arena_refcell.borrow().size(), 0);

    let mut interpreter = PatternInterpreter::new_with_refcell(
        head_index,
        arenas,
        &logging_scheduler,
        &frame_arena_refcell,
    );
    test_setup.setup_interpreter(&mut interpreter);

    // Advance the interpreter to the start position.
    // First, ignore all possible played notes before the start position.
    logging_scheduler
        .borrow_mut()
        .set_inner_enabled(false);

    interpreter
        .update_time(start_time)
        .unwrap_or_else(|err| {
            panic!("Encountered error when updating start time to {start_time:?}: {err:?}")
        });
    debug_assert_eq!(frame_arena_refcell.borrow().size(), 0,);

    logging_scheduler
        .borrow_mut()
        .set_inner_enabled(true);

    // Enable printing log messages, if desired.
    logging_scheduler
        .borrow_mut()
        .set_logging(true);
    let expect_note_unit = |expectation: ScheduledExpectation| {
        let ScheduledExpectation { start_time, duration, note_unit } =
            expectation;
        let sound_unit = SoundUnit::new(note_unit, duration);
        let add_expectation = |borrowed_scheduler: &mut MockUnitScheduler| {
            let abs_diff = |x: CycleTime, y: CycleTime| {
                if x <= y { y - x } else { x - y }
            };
            let approx_eq_start_time =
                predicate::function(move |time: &CycleTime| {
                    abs_diff(start_time, *time) <= CycleTime::EPSILON
                });
            borrowed_scheduler
                .expect_add()
                .with(predicate::eq(sound_unit.clone()), approx_eq_start_time)
                .once()
                .return_const(());
        };
        with_mock_scheduler(&add_expectation)
    };

    for borrow_scheduled_actions in expected_schedule_actions {
        let (next_cycle_time, expectations) = borrow_scheduled_actions.borrow();
        expectations
            .clone()
            .into_iter()
            .for_each(|e| expect_note_unit(e.borrow().clone()));
        interpreter
            .update_time(*next_cycle_time)
            .unwrap_or_else(|err| {
                panic!("Encountered error when updating to time {next_cycle_time:?}: {err:?}")
            });

        with_mock_scheduler(&|player| player.checkpoint());
    }

    debug_assert_eq!(frame_arena.size(), 0);
}
