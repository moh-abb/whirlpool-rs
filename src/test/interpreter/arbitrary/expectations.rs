use core::fmt::Debug;
use core::iter::once;
use core::num::NonZeroU8;
use core::num::NonZeroU16;

use proptest::prelude::Strategy;
use proptest::test_runner::Reason;

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::test_play_multiple;
use crate::ast::time::props::ElemProps;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::Vec;
use crate::test::examples::arbitrary::time::arb_cycle_time;
use crate::test::examples::arbitrary::time::arb_positive_cycle_time;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::sequence::NoteSequence;

fn multiple_expectations<T: Debug, Iter: Iterator<Item = ElemProps<T>>>(
    interval: CycleInterval,
    total: ElemProps<impl Fn() -> Iter>,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    mut get_elem_expectations: impl FnMut(
        &T,
        CycleInterval,
        CycleTime,
        CycleTime,
    ) -> NoteSequence,
) -> NoteSequence {
    assert_ne!(total.sim_duration, CycleTime::ZERO);
    assert_ne!(total.played_duration, CycleTime::ZERO);

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
        total,
        is_fast,
        offset,
        multiplier,
        append_sequence,
    );
    result_sequence
}

fn timed_step_iter<'a>(
    multiple: &'a Multiple<TimedStep>,
    arenas: &'a impl PatternArenas,
) -> impl Iterator<Item = TimedStep> + 'a {
    multiple
        .iter(arenas.get_timed_step_chain_arena())
        .filter_map(|timed_step| {
            arenas
                .get_timed_step_arena()
                .map(timed_step, Clone::clone)
                .ok()
        })
}

pub fn pattern_expectations(
    pattern: Index<Pattern>,
    arenas: &impl PatternArenas,
    interval: CycleInterval,
    offset: CycleTime,
    multiplier: CycleTime,
) -> ArenaResult<NoteSequence> {
    let cloned_pattern = arenas
        .get_pattern_arena()
        .map(pattern, Clone::clone)?;

    let result = match &cloned_pattern {
        Pattern::Cat(multiple) | Pattern::Seq(multiple) => {
            let length = CycleTime::from_int(i32::from(multiple.length()));
            let is_fast = matches!(cloned_pattern, Pattern::Seq(_));
            multiple_expectations(
                interval,
                ElemProps {
                    elem: || {
                        multiple
                            .iter(arenas.get_pattern_chain_arena())
                            .map(|elem| ElemProps {
                                elem,
                                sim_duration: CycleTime::ONE,
                                played_duration: CycleTime::ONE,
                            })
                    },
                    sim_duration: length,
                    played_duration: length,
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
            let total_cycle_length = timed_step_iter(&multiple, arenas)
                .map(|TimedStep(played_dur, _)| played_dur)
                .sum();
            let sim_duration =
                CycleTime::from_int(i32::from(multiple.length()));
            let elems_with_props = || {
                timed_step_iter(&multiple, arenas).map(
                    |TimedStep(proportion, elem)| ElemProps {
                        elem,
                        sim_duration: CycleTime::ONE,
                        played_duration: (proportion * sim_duration)
                            / total_cycle_length,
                    },
                )
            };
            // Recalculate the played duration due to rounding errors.
            let played_duration = elems_with_props()
                .map(|props| props.played_duration)
                .sum();

            assert_ne!(played_duration, CycleTime::ZERO);
            multiple_expectations(
                interval,
                ElemProps {
                    elem: elems_with_props,
                    sim_duration,
                    played_duration,
                },
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
        Pattern::Arrange(multiple) => {
            let played_duration = timed_step_iter(&multiple, arenas)
                .map(|TimedStep(played_dur, _)| played_dur)
                .sum();
            multiple_expectations(
                interval,
                ElemProps {
                    elem: || {
                        timed_step_iter(&multiple, arenas).map(
                            |TimedStep(played_duration, elem)| ElemProps {
                                elem,
                                sim_duration: played_duration,
                                played_duration,
                            },
                        )
                    },
                    sim_duration: played_duration,
                    played_duration,
                },
                false,
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
        Pattern::Note(note_unit) => multiple_expectations(
            interval,
            ElemProps {
                elem: || {
                    once(ElemProps {
                        elem: note_unit,
                        sim_duration: CycleTime::ONE,
                        played_duration: CycleTime::ONE,
                    })
                },
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            },
            false,
            offset,
            multiplier,
            |elem, sim_interval, sim_offset, sim_multiplier| {
                assert_eq!(elem, &note_unit);
                let played_note =
                    if sim_interval.start() == sim_interval.start().floor() {
                        vec![ScheduledExpectation {
                            start_time: (sim_interval.start() + sim_offset)
                                / sim_multiplier,
                            duration: sim_multiplier.recip(),
                            note_unit: note_unit.clone(),
                        }]
                    } else {
                        Vec::new()
                    };
                NoteSequence {
                    interval,
                    offset,
                    multiplier,
                    expected: played_note,
                }
            },
        ),
        Pattern::Silence => {
            NoteSequence { interval, offset, multiplier, expected: Vec::new() }
        }
    };
    Ok(result)
}

const MAX_START_TIME: CycleTime = CycleTime::from_int(2048);
const MAX_DURATION: CycleTime = CycleTime::from_int(40);

/// Returns an arbitrary end time, offset and multiplier.
pub fn arb_interval_offset_and_multiplier()
-> impl Strategy<Value = (CycleInterval, CycleTime, CycleTime)> {
    let arb_start_time = arb_positive_cycle_time::<NonZeroU16>().prop_filter(
        Reason::from("Start time should be at most {MAX_START_TIME:?}"),
        |time| time <= &MAX_START_TIME,
    );
    let arb_duration = arb_positive_cycle_time::<NonZeroU8>().prop_filter(
        Reason::from("Duration should be at most {MAX_DURATION:?}"),
        |time| time <= &MAX_DURATION,
    );
    let arb_interval =
        (arb_start_time, arb_duration).prop_map(|(start, duration)| {
            CycleInterval::new(start, start + duration)
        });
    let arb_multiplier = arb_positive_cycle_time::<NonZeroU8>();
    (arb_interval, arb_cycle_time(), arb_multiplier)
}
