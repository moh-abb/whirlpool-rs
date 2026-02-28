use core::num::NonZeroU16;

use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::test_runner::Reason;

use crate::alloc_types::Vec;
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
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::StrategyWithArena;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;

const MAX_END_TIME: CycleTime = CycleTime::from_int(2048);

pub fn unit_sequence_with_start(
    head: Index<Pattern>,
    note_unit: NoteUnit,
    start_time: CycleTime,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> NoteSequence {
    let mut scheduled_expectations = Vec::<ScheduledExpectation>::new();
    let aligned_start_time = start_time.round_up_to_nearest(CycleTime::ONE);
    let aligned_end_time = end_time.round_up_to_nearest(CycleTime::ONE);
    let unit_duration = multiplier.recip();

    let mut current = aligned_start_time;
    while current < aligned_end_time {
        let expectation = ScheduledExpectation {
            start_time: current.mul(unit_duration).add(offset),
            duration: unit_duration,
            note_unit,
        };

        scheduled_expectations.push(expectation);
        current = current.add(CycleTime::ONE);
    }

    NoteSequence {
        head,
        offset,
        multiplier,
        expected: vec![(end_time, scheduled_expectations.clone())],
    }
}

pub fn chunked_unit_sequence(
    alloc_unit: impl FnOnce(Pattern) -> ArenaResult<Index<Pattern>>,
    note_unit: NoteUnit,
    chunk_count: NonZeroU16,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    // Play intervals at sequences with times:
    // (end_time * 1) / (chunk_count + 1),
    // (end_time * 2) / (chunk_count + 1),
    // ...,
    // (end_time * chunk_count) / (chunk_count + 1),
    // end_time.
    let next_times = (1..=chunk_count.get())
        .map(i32::from)
        .map(CycleTime::from_int)
        .map(|chunk_index| {
            let chunk_count_as_time =
                CycleTime::from_int(chunk_count.get().into());
            let interpreter_time = end_time
                .mul(chunk_index)
                .div(chunk_count_as_time.add(CycleTime::ONE));
            interpreter_time
        })
        .chain(Some(end_time));

    let head = alloc_unit(Pattern::Note(note_unit))?;

    let mut result = NoteSequence {
        head: head.clone(),
        offset,
        multiplier,
        expected: Vec::new(),
    };

    let mut current_time = CycleTime::ZERO;
    for next_time in next_times {
        let chunk_sequence = unit_sequence_with_start(
            head.clone(),
            note_unit,
            current_time,
            next_time,
            offset,
            multiplier,
        );
        assert_eq!(chunk_sequence.head, head);
        assert_eq!(chunk_sequence.multiplier, multiplier);
        assert_eq!(chunk_sequence.offset, offset);
        assert_eq!(chunk_sequence.expected.len(), 1);
        assert_eq!(chunk_sequence.expected[0].0, next_time);

        // Append chunk to total expected.
        result
            .expected
            .extend(chunk_sequence.expected);
        current_time = next_time;
    }

    Ok(result)
}

pub fn unit_sequence(
    alloc_unit: impl FnOnce(Pattern) -> ArenaResult<Index<Pattern>>,
    note_unit: NoteUnit,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    chunked_unit_sequence(
        alloc_unit,
        note_unit,
        NonZeroU16::new(1).unwrap(),
        end_time,
        offset,
        multiplier,
    )
}

struct UnitSequenceStrategy;
impl StrategyWithArena<NoteSequence> for UnitSequenceStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        unit_sequence_strategy_args().prop_map(
            |(note_unit, end_time, offset, multiplier)| {
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
            },
        )
    }
}

const MAX_CHUNK_COUNT: NonZeroU16 = NonZeroU16::new(5000).unwrap();

fn arb_chunk_count() -> impl Strategy<Value = NonZeroU16> {
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
                    let alloc_unit = |pattern: Pattern| {
                        arenas
                            .get_pattern_arena()
                            .alloc(pattern)
                    };
                    chunked_unit_sequence(
                        alloc_unit,
                        note_unit,
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
