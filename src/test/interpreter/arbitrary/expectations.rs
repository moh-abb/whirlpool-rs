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
use crate::interpreter::elements::play_multiple as test_play_multiple;
use crate::interpreter::elements::timed_step_total_cycle_length;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::Vec;
use crate::test::examples::arbitrary::time::arb_cycle_time;
use crate::test::examples::arbitrary::time::arb_positive_cycle_time;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::sequence::NoteSequence;

fn multiple_expectations<
    T: Debug,
    Iter: Iterator<Item = Result<ElemProps<T>, PatternInterpreterError>>,
>(
    interval: CycleInterval,
    total: ElemProps<impl Fn() -> Iter>,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    mut get_elem_expectations: impl FnMut(
        PlayElemArgs<'_, T>,
    ) -> Result<
        NoteSequence,
        PatternInterpreterError,
    >,
) -> Result<NoteSequence, PatternInterpreterError> {
    assert_ne!(total.sim_duration, CycleTime::ZERO);
    assert_ne!(total.played_duration, CycleTime::ZERO);

    let mut result_sequence =
        Ok(NoteSequence { interval, offset, multiplier, expected: Vec::new() });
    let append_sequence = |args: PlayElemArgs<'_, T>| {
        let elem_sequence = get_elem_expectations(args);
        // Note that the simulated offsets may be different due to the
        // alteration of elements, and the simulated multiplier is different
        // if the sequence is fast.
        match (&mut result_sequence, &elem_sequence) {
            (Ok(sequence), Ok(elem_sequence)) => {
                sequence
                    .expected
                    .extend(elem_sequence.expected.clone());
            }
            (_, Err(PatternInterpreterError::ArenaErr(e))) => {
                // The error will be propagated in the return value.
                result_sequence =
                    Err(PatternInterpreterError::ArenaErr(e.clone()));
            }
            (_, Err(err @ PatternInterpreterError::OverflowErr(_))) => {
                return Err(err.clone());
            }
            (Err(_), Ok(_)) => {
                // The error has already been set.
            }
        }
        Ok(())
    };
    test_play_multiple(
        interval,
        total,
        is_fast,
        offset,
        multiplier,
        append_sequence,
    )?;
    result_sequence
}

fn timed_step_iter<'a>(
    multiple: &'a Multiple<TimedStep>,
    arenas: &'a impl PatternArenas,
) -> impl Iterator<Item = ArenaResult<TimedStep>> + 'a {
    multiple
        .checked_iter(arenas.get_timed_step_chain_arena())
        .map(|opt_timed_step| {
            arenas
                .get_timed_step_arena()
                .map(opt_timed_step?, Clone::clone)
        })
}

pub fn pattern_expectations(
    pattern: Index<Pattern>,
    arenas: &impl PatternArenas,
    interval: CycleInterval,
    offset: CycleTime,
    multiplier: CycleTime,
) -> Result<NoteSequence, PatternInterpreterError> {
    let cloned_pattern = arenas
        .get_pattern_arena()
        .map(pattern, Clone::clone)?;

    match &cloned_pattern {
        Pattern::Cat(multiple) | Pattern::Seq(multiple) => {
            let length =
                CycleTime::checked_from_int(i32::from(multiple.length()))?;
            let is_fast = matches!(cloned_pattern, Pattern::Seq(_));
            multiple_expectations(
                interval,
                ElemProps {
                    elem: || {
                        multiple
                            .iter(arenas.get_pattern_chain_arena())
                            .map(|elem| {
                                Ok(ElemProps {
                                    elem,
                                    sim_duration: CycleTime::ONE,
                                    played_duration: CycleTime::ONE,
                                })
                            })
                    },
                    sim_duration: length,
                    played_duration: length,
                },
                is_fast,
                offset,
                multiplier,
                |args| {
                    pattern_expectations(
                        args.elem.clone(),
                        arenas,
                        args.interval,
                        args.offset,
                        args.multiplier,
                    )
                },
            )
        }
        Pattern::Stack(multiple) => multiple
            .iter(arenas.get_pattern_chain_arena())
            .map(|elem| {
                pattern_expectations(elem, arenas, interval, offset, multiplier)
            })
            .reduce(|opt_x, opt_y| {
                let mut x = opt_x?;
                let y = opt_y?;
                assert_eq!(x.interval, y.interval);
                assert_eq!(x.offset, y.offset);
                assert_eq!(x.multiplier, y.multiplier);
                x.expected.extend(y.expected);
                Ok(x)
            })
            .unwrap(),
        Pattern::TimeCat(multiple) => {
            let total_cycle_length =
                timed_step_total_cycle_length(&multiple, arenas)?;
            let sim_duration =
                CycleTime::checked_from_int(i32::from(multiple.length()))?;
            let elems_with_props = || {
                timed_step_iter(&multiple, arenas).map(|opt_timed_step| {
                    let TimedStep(proportion, elem) = opt_timed_step?;
                    Result::<_, PatternInterpreterError>::Ok(ElemProps {
                        elem,
                        sim_duration: CycleTime::ONE,
                        played_duration: proportion
                            .mul(sim_duration)?
                            .div(total_cycle_length)?,
                    })
                })
            };
            // Recalculate the played duration due to rounding errors.
            let played_duration =
                elems_with_props().try_fold(CycleTime::ZERO, |acc, x| {
                    Result::<_, PatternInterpreterError>::Ok(
                        acc.add(x?.played_duration)?,
                    )
                })?;

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
                |args| {
                    pattern_expectations(
                        args.elem.clone(),
                        arenas,
                        args.interval,
                        args.offset,
                        args.multiplier,
                    )
                },
            )
        }
        Pattern::Arrange(multiple) => {
            let played_duration =
                timed_step_total_cycle_length(&multiple, arenas)?;
            let elems_with_props = || {
                timed_step_iter(&multiple, arenas).map(|opt_timed_step| {
                    let TimedStep(played_duration, elem) = opt_timed_step?;
                    Result::<_, PatternInterpreterError>::Ok(ElemProps {
                        elem,
                        sim_duration: played_duration,
                        played_duration,
                    })
                })
            };
            multiple_expectations(
                interval,
                ElemProps {
                    elem: elems_with_props,
                    sim_duration: played_duration,
                    played_duration,
                },
                false,
                offset,
                multiplier,
                |args| {
                    pattern_expectations(
                        args.elem.clone(),
                        arenas,
                        args.interval,
                        args.offset,
                        args.multiplier,
                    )
                },
            )
        }
        Pattern::Note(note_unit) => multiple_expectations(
            interval,
            ElemProps {
                elem: || {
                    once(Ok(ElemProps {
                        elem: note_unit,
                        sim_duration: CycleTime::ONE,
                        played_duration: CycleTime::ONE,
                    }))
                },
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            },
            false,
            offset,
            multiplier,
            |args| {
                assert_eq!(args.elem, &note_unit);
                let start = args.interval.start();
                let floor_start = start.floor()?;
                let mut expected_note = Vec::new();
                if start == floor_start {
                    expected_note.push(ScheduledExpectation {
                        start_time: args
                            .interval
                            .start()
                            .add(args.offset)?
                            .div(args.multiplier)?,
                        duration: args.multiplier.recip()?,
                        note_unit: note_unit.clone(),
                    });
                }
                Ok(NoteSequence {
                    interval,
                    offset,
                    multiplier,
                    expected: expected_note,
                })
            },
        ),
        Pattern::Silence => Ok(NoteSequence {
            interval,
            offset,
            multiplier,
            expected: Vec::new(),
        }),
    }
}

const MAX_START_TIME: CycleTime = CycleTime::unwrapped_from_int(1 << 9);
const MAX_DURATION: CycleTime = CycleTime::unwrapped_from_int(1 << 5);
const MAX_MULTIPLIER: CycleTime = CycleTime::unwrapped_from_int(1 << 4);

/// Returns an arbitrary end time, offset and multiplier.
/// The maximum start time, duration and multiplier are chosen with the maximum
/// patten size to avoid the overflow case.
/// With a multiplier of 2^4, pattern size of ~64 = 2^6,
/// and maximum start time of 2^9
/// (with the duration substantially less than the start time),
/// we would expect to require 4 + 6 + 9 = 19 bits of precision.
/// This should fit within the 20 integer bits provided by [CycleTime].
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
    let arb_interval = (arb_start_time, arb_duration).prop_filter_map(
        Reason::from("Overflow error"),
        |(start, duration)| {
            Some(CycleInterval::new(start, start.add(duration).ok()?))
        },
    );
    let arb_multiplier = arb_positive_cycle_time::<NonZeroU8>().prop_filter(
        Reason::from("Duration should be at most {MAX_MULTIPLIER:?}"),
        |time| time <= &MAX_MULTIPLIER,
    );
    (arb_interval, arb_cycle_time(), arb_multiplier)
}
