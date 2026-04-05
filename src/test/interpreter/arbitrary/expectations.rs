use core::fmt::Debug;
use core::iter::repeat;
use core::num::NonZeroU16;

use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::test_runner::Reason;

use crate::alloc_types::Vec;
use crate::arena::Arena;
use crate::arena::error::ArenaResult;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::test_play_multiple;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::ast::time::CycleTimeInterval;
use crate::structures::index::Index;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::pattern::arbitrary::time::arb_cycle_time;
use crate::test::pattern::arbitrary::time::arb_positive_cycle_time;

fn interval_whole_times(
    interval: CycleTimeInterval,
) -> impl Iterator<Item = CycleTime> {
    let start = interval.start().ceil().to_int();
    let end = interval.end().floor().to_int();
    (start..=end).map(CycleTime::from_int)
}

fn unit_expectations(
    interval: CycleTimeInterval,
    note_unit: NoteUnit,
    offset: CycleTime,
    multiplier: CycleTime,
) -> NoteSequence {
    let unit_duration = multiplier.recip();
    let expected = interval_whole_times(interval)
        .map(|start_time| ScheduledExpectation {
            start_time,
            duration: unit_duration,
            note_unit,
        })
        .collect();
    NoteSequence { interval, offset, multiplier, expected }
}

fn multiple_expectations<T: Debug, Iter: Iterator<Item = (CycleTime, T)>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    mut get_elem_expectations: impl FnMut(
        &T,
        CycleTimeInterval,
        CycleTime,
        CycleTime,
    ) -> NoteSequence,
) -> NoteSequence {
    let mut result_sequence =
        NoteSequence { interval, offset, multiplier, expected: Vec::new() };
    let append_sequence =
        |elem: &T, sim_interval, sim_offset, sim_multiplier| {
            let elem_sequence = get_elem_expectations(
                elem,
                sim_interval,
                sim_offset,
                sim_multiplier,
            );
            // Note that the simulated offsets may be different due to the
            // alteration of elements, and the simulated multiplier is different
            // if the sequence is fast.
            result_sequence
                .expected
                .extend(elem_sequence.expected);
        };
    test_play_multiple(
        interval,
        length,
        elements,
        is_fast,
        offset,
        multiplier,
        append_sequence,
    );
    result_sequence
}

pub fn pattern_expectations(
    pattern: Index<Pattern>,
    arenas: &impl PatternArenas,
    interval: CycleTimeInterval,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    let cloned_pattern = arenas
        .get_pattern_arena()
        .inspect(pattern, Clone::clone)?;
    let result = match cloned_pattern.clone() {
        Pattern::Cat(multiple) | Pattern::Seq(multiple) => {
            let is_fast = matches!(cloned_pattern, Pattern::Seq(_));
            multiple_expectations(
                interval,
                CycleTime::from_int(i32::from(multiple.length())),
                || {
                    repeat(CycleTime::ONE)
                        .zip(multiple.iter(arenas.get_pattern_chain_arena()))
                },
                is_fast,
                offset,
                multiplier,
                |elem, sim_interval, sim_offset, sim_multiplier| {
                    pattern_expectations(
                        elem.clone(),
                        arenas,
                        sim_interval,
                        sim_offset,
                        sim_multiplier,
                    )
                    .unwrap()
                },
            )
        }
        Pattern::Stack(multiple) => multiple
            .iter(arenas.get_pattern_chain_arena())
            .filter_map(|elem| {
                pattern_expectations(elem, arenas, interval, offset, multiplier)
                    .ok()
            })
            .reduce(|mut x, y| {
                assert_eq!(x.interval, y.interval);
                assert_eq!(x.offset, y.offset);
                assert_eq!(x.multiplier, y.multiplier);
                x.expected.extend(y.expected);
                x
            })
            .unwrap(),
        Pattern::TimeCat(multiple) => {
            let elems_with_durations = || {
                multiple
                    .iter(arenas.get_timed_step_chain_arena())
                    .filter_map(|timed_step| {
                        arenas
                            .get_timed_step_arena()
                            .inspect(timed_step, Clone::clone)
                            .map(|TimedStep(unit, pattern)| (unit, pattern))
                            .ok()
                    })
            };
            let length = elems_with_durations()
                .map(|(dur, _)| dur)
                .fold(CycleTime::ZERO, CycleTime::add);
            assert_ne!(length, CycleTime::ZERO);
            multiple_expectations(
                interval,
                length,
                elems_with_durations,
                true,
                offset,
                multiplier,
                |elem, sim_interval, sim_offset, sim_multiplier| {
                    pattern_expectations(
                        elem.clone(),
                        arenas,
                        sim_interval,
                        sim_offset,
                        sim_multiplier,
                    )
                    .unwrap()
                },
            )
        }
        Pattern::Note(note_unit) => {
            unit_expectations(interval, note_unit, offset, multiplier)
        }
        Pattern::Silence => {
            NoteSequence { interval, offset, multiplier, expected: Vec::new() }
        }
    };
    Ok(result)
}

const MAX_END_TIME: CycleTime = CycleTime::from_int(2048);

/// Returns an arbitrary end time, offset and multiplier.
pub fn arb_end_offset_and_multiplier()
-> impl Strategy<Value = (CycleTime, CycleTime, CycleTime)> {
    let arb_end_time = arb_positive_cycle_time().prop_filter(
        Reason::from("End time should be at most {MAX_END_TIME:?}"),
        |time| time <= &MAX_END_TIME,
    );
    let arb_multiplier = any::<NonZeroU16>()
        .prop_map(NonZeroU16::get)
        .prop_map(i32::from)
        .prop_map(CycleTime::from_int);
    (arb_end_time, arb_cycle_time(), arb_multiplier)
}
