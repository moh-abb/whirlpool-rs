use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::test_runner::Reason;

use crate::arena::Arena;
use crate::arena::error::ArenaResult;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::Interpreter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::structures::index::Index;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::TestSetupStrategy;
use crate::test::interpreter::arbitrary::time::arb_cycle_time;
use crate::test::interpreter::arbitrary::time::arb_positive_cycle_time;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::interpreter::test_expectations_with_interpreter_setup;
use crate::test::pattern::arbitrary::ArenasTo;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::StrategyWithArena;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;

const MAX_END_TIME: CycleTime = CycleTime::from_int(2048);

pub fn unit_sequence(
    alloc_unit: impl FnOnce(Pattern) -> ArenaResult<Index<Pattern>>,
    note_unit: NoteUnit,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    let mut expected = Vec::<ScheduledExpectation>::new();
    let aligned_end_time = end_time.round_up_to_nearest(CycleTime::ONE);
    let unit_duration = multiplier.recip();

    let mut current = CycleTime::ZERO;
    while current < aligned_end_time {
        let expectation = ScheduledExpectation {
            start_time: current.mul(unit_duration).add(offset),
            duration: unit_duration,
            note_unit,
        };

        expected.push(expectation);
        current = current.add(CycleTime::ONE);
    }

    let head = alloc_unit(Pattern::Note(note_unit))?;
    Ok(NoteSequence {
        head,
        offset,
        multiplier,
        end_time,
        expected: expected.clone(),
    })
}

struct UnitSequenceStrategy;
impl StrategyWithArena<NoteSequence> for UnitSequenceStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        let arb_end_time = arb_positive_cycle_time().prop_filter(
            Reason::from("End time should be at most {MAX_END_TIME:?}"),
            |time| time <= &MAX_END_TIME,
        );
        let arbitrary_args = (
            any::<NoteUnit>(),
            arb_end_time,
            arb_cycle_time(),
            arb_positive_cycle_time(),
        );
        arbitrary_args.prop_map(|(note_unit, end_time, offset, multiplier)| {
            ArenasTo::new(move |arenas: &Arenas| {
                let alloc_unit = |pattern: Pattern| {
                    arenas
                        .get_pattern_arena()
                        .alloc(pattern)
                };
                unit_sequence(
                    alloc_unit, note_unit, end_time, offset, multiplier,
                )
            })
        })
    }
}

struct UnitSequenceTestSetup {
    offset: CycleTime,
    multiplier: CycleTime,
}
impl TestSetupStrategy for UnitSequenceTestSetup {
    fn setup_interpreter<Arenas, Player, BorrowAdapter>(
        self,
        interpreter: &mut Interpreter<Arenas, Player, BorrowAdapter>,
    ) {
        interpreter.set_multiplier(self.multiplier);
        interpreter.set_offset(self.offset);
    }
}

struct PlayCompleteTimeInterval;
impl ArenaTest<NoteSequence> for PlayCompleteTimeInterval {
    fn run(arenas: &impl PatternArenas, sequence: NoteSequence) {
        let complete_interval_expectations =
            [(sequence.end_time, sequence.expected.as_slice())];
        test_expectations_with_interpreter_setup(
            arenas,
            sequence.head,
            &complete_interval_expectations,
            UnitSequenceTestSetup {
                offset: sequence.offset,
                multiplier: sequence.multiplier,
            },
        );
    }
}

#[test]
fn can_play_complete_time_interval_once() {
    with_regenerated_arenas::<
        _,
        PlayCompleteTimeInterval,
        GrowableArenas,
        UnitSequenceStrategy,
    >()
}
