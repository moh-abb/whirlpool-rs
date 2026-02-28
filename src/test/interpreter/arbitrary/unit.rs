use core::num::NonZeroU16;

use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::test_runner::Reason;

use crate::arena::Arena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::Interpreter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::test::interpreter::TestSetupStrategy;
use crate::test::interpreter::arbitrary::repeated::chunked_repeated_unit;
use crate::test::interpreter::arbitrary::repeated::repeated_unit;
use crate::test::interpreter::arbitrary::time::arb_cycle_time;
use crate::test::interpreter::arbitrary::time::arb_positive_cycle_time;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::interpreter::test_expectations_with_interpreter_setup;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::StrategyWithArena;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;

const MAX_END_TIME: CycleTime = CycleTime::from_int(2048);

struct UnitSequenceStrategy;
impl StrategyWithArena<NoteSequence> for UnitSequenceStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        unit_sequence_strategy_args().prop_map(
            |(note_unit, end_time, offset, multiplier)| {
                ArenasTo::new(move |arenas: &Arenas| {
                    let head = arenas
                        .get_pattern_arena()
                        .alloc(Pattern::Note(note_unit))?;
                    repeated_unit(
                        head,
                        |_| note_unit,
                        end_time,
                        offset,
                        multiplier,
                    )
                })
            },
        )
    }
}

const MAX_CHUNK_COUNT: NonZeroU16 = NonZeroU16::new(5000).unwrap();

pub fn arb_chunk_count() -> impl Strategy<Value = NonZeroU16> {
    any::<NonZeroU16>().prop_filter(
        Reason::from("Chunk count should be less than {MAX_CHUNK_COUNT}"),
        |chunk_count| chunk_count <= &MAX_CHUNK_COUNT,
    )
}

struct ChunkedUnitSequenceStrategy;
impl StrategyWithArena<NoteSequence> for ChunkedUnitSequenceStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        (arb_chunk_count(), unit_sequence_strategy_args()).prop_map(
            |(chunk_count, (note_unit, end_time, offset, multiplier))| {
                ArenasTo::new(move |arenas: &Arenas| {
                    let head = arenas
                        .get_pattern_arena()
                        .alloc(Pattern::Note(note_unit))?;
                    chunked_repeated_unit(
                        head,
                        |_| note_unit,
                        chunk_count,
                        end_time,
                        offset,
                        multiplier,
                    )
                })
            },
        )
    }
}

/// Returns an arbitrary note unit, end time, offset and multiplier.
fn unit_sequence_strategy_args()
-> impl Strategy<Value = (NoteUnit, CycleTime, CycleTime, CycleTime)> {
    let arb_end_time = arb_positive_cycle_time().prop_filter(
        Reason::from("End time should be at most {MAX_END_TIME:?}"),
        |time| time <= &MAX_END_TIME,
    );
    (
        any::<NoteUnit>(),
        arb_end_time,
        arb_cycle_time(),
        arb_positive_cycle_time(),
    )
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

struct PlayNoteSequence;
impl ArenaTest<NoteSequence> for PlayNoteSequence {
    fn run(arenas: &impl PatternArenas, sequence: NoteSequence) {
        test_expectations_with_interpreter_setup(
            arenas,
            sequence.head,
            &sequence.expected,
            UnitSequenceTestSetup {
                offset: sequence.offset,
                multiplier: sequence.multiplier,
            },
        );
    }
}

#[test]
fn can_play_complete_time_interval() {
    with_regenerated_arenas::<
        _,
        PlayNoteSequence,
        GrowableArenas,
        UnitSequenceStrategy,
    >()
}

#[test]
fn can_play_chunked_time_interval() {
    with_regenerated_arenas::<
        _,
        PlayNoteSequence,
        GrowableArenas,
        ChunkedUnitSequenceStrategy,
    >()
}
