use core::num::NonZeroU16;

use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::test_runner::Reason;

use crate::alloc_types::Vec;
use crate::arena::error::ArenaResult;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::structures::index::Index;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::arbitrary::time::arb_cycle_time;
use crate::test::interpreter::arbitrary::time::arb_positive_cycle_time;
use crate::test::interpreter::sequence::NoteSequence;

fn repeated_unit_with_start(
    head: Index<Pattern>,
    get_unit: &impl Fn(CycleTime) -> NoteUnit,
    start_time: CycleTime,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> NoteSequence {
    let mut scheduled_expectations = Vec::<ScheduledExpectation>::new();
    let aligned_start_time = start_time.ceil();
    let aligned_end_time = end_time.ceil();
    let unit_duration = multiplier.recip();

    let mut current = aligned_start_time;
    while current < aligned_end_time {
        let expectation = ScheduledExpectation {
            start_time: offset.add(current.div(multiplier)),
            duration: unit_duration,
            note_unit: get_unit(current),
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

pub fn chunked_repeated_unit(
    head: Index<Pattern>,
    get_unit: impl Fn(CycleTime) -> NoteUnit,
    chunk_count: NonZeroU16,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    // Play intervals at sequences with times:
    // (end_time * 1) / chunk_count,
    // (end_time * 2) / chunk_count,
    // ...,
    // (end_time * (chunk_count - 1)) / chunk_count,
    // end_time.
    let next_times = (1..chunk_count.get())
        .map(i32::from)
        .map(CycleTime::from_int)
        .map(|chunk_index| {
            let chunk_count_as_time =
                CycleTime::from_int(chunk_count.get().into());
            let interpreter_time = end_time
                .mul(chunk_index)
                .div(chunk_count_as_time);
            interpreter_time
        })
        .chain(Some(end_time));

    let mut result = NoteSequence {
        head: head.clone(),
        offset,
        multiplier,
        expected: Vec::new(),
    };

    let mut current_time = CycleTime::ZERO;
    for next_time in next_times {
        let chunk_sequence = repeated_unit_with_start(
            head.clone(),
            &get_unit,
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

pub fn repeated_unit(
    head: Index<Pattern>,
    get_unit: impl Fn(CycleTime) -> NoteUnit,
    end_time: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    chunked_repeated_unit(
        head,
        get_unit,
        NonZeroU16::new(1).unwrap(),
        end_time,
        offset,
        multiplier,
    )
}

const MAX_END_TIME: CycleTime = CycleTime::from_int(2048);

/// Returns an arbitrary end time, offset and multiplier.
pub fn arb_end_offset_and_multiplier()
-> impl Strategy<Value = (CycleTime, CycleTime, CycleTime)> {
    let arb_end_time = arb_positive_cycle_time().prop_filter(
        Reason::from("End time should be at most {MAX_END_TIME:?}"),
        |time| time <= &MAX_END_TIME,
    );
    (arb_end_time, arb_cycle_time(), arb_positive_cycle_time())
}

const MAX_CHUNK_COUNT: NonZeroU16 = NonZeroU16::new(5000).unwrap();

pub fn arb_chunk_count() -> impl Strategy<Value = NonZeroU16> {
    any::<NonZeroU16>().prop_filter(
        Reason::from("Chunk count should be less than {MAX_CHUNK_COUNT}"),
        |chunk_count| chunk_count <= &MAX_CHUNK_COUNT,
    )
}
