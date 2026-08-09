use core::fmt::Debug;
use core::iter;
use core::ops::RangeInclusive;

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::Multiple;

pub type PlayElementsResult<T> = Result<T, PatternInterpreterError>;

#[inline]
fn play_intersection<T: Debug>(
    elem: T,
    rep_interval: CycleInterval,
    rep_intersection: CycleInterval,
    sim_interval: CycleInterval,
    elem_offset: CycleTime,
    elem_multiplier: CycleTime,
) -> PlayElementsResult<Option<PlayElemArgs<T>>> {
    // `rep_intersection` should fit completely inside `rep_interval`.
    debug_assert_eq!(
        rep_intersection.intersection(rep_interval),
        Some(rep_intersection)
    );

    let opt_sim_intersection =
        sim_interval.lerp_interval(rep_intersection, rep_interval)?;
    let Some(sim_intersection) = opt_sim_intersection else {
        // Due to fixed point arithmetic errors, the intersection is too small
        // to consider.
        return Ok(None);
    };

    // `sim_intersection` should fit completely inside `sim_interval`.
    debug_assert_eq!(
        sim_intersection.intersection(sim_interval),
        Some(sim_intersection),
    );

    Ok(Some(PlayElemArgs {
        elem,
        interval: sim_intersection,
        offset: elem_offset,
        multiplier: elem_multiplier,
    }))
}

struct OuterIterData {
    played_start: CycleTime,
    interval: CycleInterval,
    total: ElemProps<()>,
    offset: CycleTime,
    total_multiplier: CycleTime,
}

type OuterIter<T, Iter> = iter::Scan<
    Iter,
    OuterIterData,
    fn(
        &mut OuterIterData,
        PlayElementsResult<ElemProps<T>>,
    ) -> Option<RepIterData<T>>,
>;

fn make_outer_iter<T, Iter>(
    interval: CycleInterval,
    total: ElemProps<Iter>,
    offset: CycleTime,
    total_multiplier: CycleTime,
) -> OuterIter<T, Iter>
where
    T: Debug + Clone,
    Iter: Iterator<Item = PlayElementsResult<ElemProps<T>>>,
{
    // Cat and similar patterns play alternating elements.
    // This is achieved by looking at the "repetitions" of each element.
    //
    // Say we have a sequence of elements { x1, x2, ..., x_{k + 1}, ..., x_n },
    // with their played lengths { p1, p2, ..., p_{k + 1}, ..., p_n }, and
    // simulated lengths { s1, s2, ..., s_{k + 1}, ..., s_n }.
    //
    // Normally, the simulated and played lengths are the same, except for
    // TimeCat where the simulated length is 1, while the played length is
    // total played length / (element played length * number of elements).
    //
    // Denote P(k) = p1 + p2 + ... + p_k and PL = total_played_duration == P(n),
    // and    S(k) = s1 + s2 + ... + s_k and SL = total_sim_duration    == S(n).
    //
    // Each unit repeats at different times:
    // x1:        played at [0, p1), [PL, PL + p1), [2*PL, 2*PL + p1), ...
    //         simulated at [0, s1), [s1, 2*s1),    [2*s1, 3*s1), ...
    // x2:        played at [P(1), P(2)), [PL + P(1), PL + P(2)),
    //                      [2*PL + P(1), 2*PL + P(2)), ...
    //         simulated at [0, s2), [s2, 2*s2), [2*s2, 3*s2), ...
    // ...
    // x_{i + 1}: played at [P(i), P(i+1)), [PL + P(i), PL + P(i+1)),
    //                      [2*PL + P(i), 2*PL + P(i+1)), ...
    //         simulated at [0, s_{i+1}), [s_{i+1}, 2*s_{i+1}),
    //                      [2*s_{i+1}, 3*s_{i+1}), ...
    // We say the nth pair of "played/simulated at" is the nth repetition,
    // for r >= 0:
    // x_{i + 1}'s r'th repetition: played at [r*PL + P(i), r*PL + P(i+1))
    //                           simulated at [r*s_{i+1}, (r+1)*s_{i+1})
    // The multiplier of any repetition of x_{i+1} is (s_{i+1} / p_{i+1}) * m_t
    // where m_t is the total multiplier.
    // (i.e., the multiplier is proportional to the ratio between the simulated
    // played lengths).
    //
    // The law that relates played and simulated times is:
    // T_p = (T_s + o) / m_s where T_p = played start time
    //                             T_s = simulated start time
    //                             o   = simulated offset
    //                             m_s = simulated multiplier
    //                                 = (s_i / p_i) * m_t (as earlier).
    // Then for element x_{i+1}, repetition i:
    // T_p = r*PL + P(i)       (by definition of a repetition earlier)
    // T_s = r*s_{i+1}         (ditto)
    // m_s = (s_i / p_i) * m_t (as defined earlier)
    // so r*PL + P(i)   = (r*s_{i+1} + o) / ((s_i / p_i) * m_t)
    //    r*s_{i+1} + o = (r*PL + P(i)) * (s_{i+1} / p_{i+1}) * m_t
    //    o             = (r*PL + P(i)) * (s_{i+1} / p_{i+1}) * m_t - r*s_{i+1}
    //                  = rep_start * elem_multiplier - rep * cur.sim_duration
    //
    // Iterate over i in [1, n].
    // INV: played_start == P(i - 1)
    let start_outer_iter = OuterIterData {
        played_start: CycleTime::ZERO,
        interval,
        offset,
        total_multiplier,
        total: ElemProps {
            elem: (),
            sim_duration: total.sim_duration,
            played_duration: total.played_duration,
        },
    };

    let scan_func =
        |state: &mut OuterIterData,
         opt_cur: PlayElementsResult<ElemProps<T>>| {
            let yield_reps = || {
                let cur = opt_cur?;
                // INV: played_start == P(i - 1)
                // elem_interval = [P(i - 1), P(i))
                let elem_interval = CycleInterval::new(
                    state.played_start,
                    state
                        .played_start
                        .add(cur.played_duration)?,
                );

                // Upper and lower bounds of rep.
                // Not all of these will be inside the played interval,
                // so we need to verify its intersection.
                let max_start_difference = state
                    .interval
                    .start()
                    .floor()?
                    .sub(elem_interval.start().ceil()?)?;
                let max_end_difference = state
                    .interval
                    .end()
                    .ceil()?
                    .sub(elem_interval.end().floor()?)?;
                let first_rep = max_start_difference
                    .div(state.total.played_duration)?
                    .floor()?;
                let last_rep = max_end_difference
                    .div(state.total.played_duration)?
                    .ceil()?;

                debug_assert!(
                    state
                        .played_start
                        .add(first_rep.mul(state.total.played_duration)?)?
                        <= state.interval.start(),
                    "first repetition to try should start on or before interval"
                );
                debug_assert!(
                    state
                        .played_start
                        .add(last_rep.mul(state.total.played_duration)?)?
                        .add(cur.played_duration)?
                        >= state.interval.end(),
                    "last repetition to try should end on or after interval"
                );

                // elem_multiplier = m_s = (s_i / p_i) * m_t
                let multiplier_scale = cur
                    .sim_duration
                    .div(cur.played_duration)?;
                let elem_multiplier =
                    multiplier_scale.mul(state.total_multiplier)?;

                // INV: rep_start = rep * PL + P(i - 1)
                let rep_start = first_rep
                    .mul(state.total.played_duration)?
                    .add(state.played_start)?;

                state.played_start = elem_interval.end();

                let outer_data_without_iter = OuterIterData {
                    played_start: state.played_start,
                    interval: state.interval,
                    total: ElemProps {
                        elem: (),
                        sim_duration: state.total.sim_duration,
                        played_duration: state.total.played_duration,
                    },
                    offset: state.offset,
                    total_multiplier: state.total_multiplier,
                };

                PlayElementsResult::Ok(RepIterData {
                    outer: outer_data_without_iter,
                    rep_start,
                    first_rep: first_rep.to_int(),
                    last_rep: last_rep.to_int(),
                    cur,
                    multiplier_scale,
                    elem_multiplier,
                })
            };

            match yield_reps() {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }
        };

    total
        .elem
        .scan(start_outer_iter, scan_func)
}

struct RepIterData<T> {
    outer: OuterIterData,
    rep_start: CycleTime,
    cur: ElemProps<T>,
    first_rep: i32,
    last_rep: i32,
    multiplier_scale: CycleTime,
    elem_multiplier: CycleTime,
}

type RepsIter<T> = iter::Scan<
    RangeInclusive<i32>,
    RepIterData<T>,
    fn(
        &mut RepIterData<T>,
        i32,
    ) -> Option<PlayElementsResult<Option<PlayElemArgs<T>>>>,
>;

fn make_reps_iter<T: Debug + Clone>(
    rep_iter_data: RepIterData<T>,
) -> RepsIter<T> {
    let rep_indices = (rep_iter_data.first_rep..=rep_iter_data.last_rep);
    let scan_func = |state: &mut RepIterData<T>, rep_index| {
        let mut yield_intersections = || {
            let rep = CycleTime::checked_from_int(rep_index)?;
            // Verify invariant for rep_start
            debug_assert_eq!(
                state.rep_start,
                rep.mul(state.outer.total.played_duration)?
                    .add(state.outer.played_start)?
            );

            // rep_interval = [rep_start, rep_start + p_i)
            let rep_interval = CycleInterval::new(
                state.rep_start,
                state
                    .rep_start
                    .add(state.cur.played_duration)?,
            );
            // Calculation detailed above.
            let rep_offset = state
                .rep_start
                .add(state.outer.offset)?
                .mul(state.multiplier_scale)?
                .sub(rep.mul(state.cur.sim_duration)?)?;

            let opt_rep_intersection =
                rep_interval.intersection(state.outer.interval);

            state.rep_start = state
                .rep_start
                .add(state.outer.total.played_duration)?;

            let Some(rep_intersection) = opt_rep_intersection else {
                return Ok(None);
            };

            // sim_interval == [r*s_i, (r+1)*s_i)
            let sim_start = rep.mul(state.cur.sim_duration)?;
            let sim_interval = CycleInterval::new(
                sim_start,
                sim_start.add(state.cur.sim_duration)?,
            );

            let opt_intersection = play_intersection(
                state.cur.elem.clone(),
                rep_interval,
                rep_intersection,
                sim_interval,
                rep_offset,
                state.elem_multiplier,
            )?;

            Ok(opt_intersection)
        };

        Some(yield_intersections())
    };

    rep_indices.scan(rep_iter_data, scan_func)
}

pub struct PlayMultiple<T, Iter>(PlayMultipleIter<T, Iter>);

type PlayMultipleIter<T, Iter> = iter::FilterMap<
    iter::FlatMap<
        OuterIter<T, Iter>,
        RepsIter<T>,
        fn(RepIterData<T>) -> RepsIter<T>,
    >,
    fn(
        PlayElementsResult<Option<PlayElemArgs<T>>>,
    ) -> Option<PlayElementsResult<PlayElemArgs<T>>>,
>;

fn make_play_multiple<T, Iter>(
    outer_iter: OuterIter<T, Iter>,
) -> PlayMultipleIter<T, Iter>
where
    T: Debug + Clone,
    Iter: Iterator<Item = PlayElementsResult<ElemProps<T>>>,
{
    let filter_args = |opt_args| match opt_args {
        Ok(Some(intersection)) => Some(Ok(intersection)),
        Ok(None) => None,
        Err(err) => Some(Err(err)),
    };

    outer_iter
        .flat_map(make_reps_iter as fn(_) -> _)
        .filter_map(filter_args as fn(_) -> _)
}

impl<T, Iter> Iterator for PlayMultiple<T, Iter>
where
    T: Debug + Clone,
    Iter: Iterator<Item = PlayElementsResult<ElemProps<T>>>,
{
    type Item = PlayElementsResult<PlayElemArgs<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn timed_step_total_cycle_length(
    multiple: &Multiple<PatternNode>,
    arenas: &impl PatternArenas,
) -> PlayElementsResult<CycleTime> {
    sum_cycle_length(
        multiple
            .checked_iter(arenas)
            .map(|opt_node_index| {
                let node = arenas
                    .get_pattern_arena()
                    .map(opt_node_index?, Clone::clone)?;
                let Pattern::TimedStep(timed_step) = node.pattern else {
                    return Err(PatternInterpreterError::ExpectedTimedStep);
                };
                Ok(timed_step)
            }),
        |TimedStep(dur, _)| dur,
    )
}

/// Verifies that the elements of an iterator have the correct simulation
/// properties (simulated and played durations) as expected.
#[inline]
#[must_use]
fn check_elem_lengths<T, Iter>(
    get_iter: impl Fn() -> Iter,
    sim_duration: CycleTime,
    played_duration: CycleTime,
) -> PlayElementsResult<()>
where
    Iter: Iterator<Item = PlayElementsResult<ElemProps<T>>>,
{
    if cfg!(debug_assertions) {
        let calculated_sim_duration =
            sum_cycle_length(get_iter(), |x| x.sim_duration)?;

        let calculated_played_duration =
            sum_cycle_length(get_iter(), |x| x.played_duration)?;

        debug_assert_eq!(calculated_sim_duration, sim_duration);
        debug_assert_eq!(calculated_played_duration, played_duration);
    }

    Ok(())
}

pub fn sum_cycle_length<T>(
    mut iter: impl Iterator<Item = PlayElementsResult<T>>,
    mut f: impl FnMut(T) -> CycleTime,
) -> PlayElementsResult<CycleTime> {
    iter.try_fold(CycleTime::ZERO, |acc, x| {
        let time = f(x?);
        Ok(acc.add(time)?)
    })
}

#[inline]
pub fn play_multiple_elements<
    T: Debug + Clone,
    Iter: Iterator<Item = PlayElementsResult<ElemProps<T>>>,
>(
    play_args: PlayElemArgs<ElemProps<impl Fn() -> Iter>>,
    is_fast: bool,
) -> PlayElementsResult<PlayMultiple<T, Iter>> {
    let PlayElemArgs { elem: total, mut interval, mut offset, mut multiplier } =
        play_args;

    check_elem_lengths(&total.elem, total.sim_duration, total.played_duration);
    if cfg!(debug_assertions) {
        let calculated_sim_duration =
            sum_cycle_length((total.elem)(), |x| x.sim_duration)?;

        let calculated_played_duration =
            sum_cycle_length((total.elem)(), |x| x.played_duration)?;

        debug_assert_eq!(calculated_sim_duration, total.sim_duration);
        debug_assert_eq!(calculated_played_duration, total.played_duration);
    }

    let total_props = ElemProps {
        elem: (total.elem)(),
        sim_duration: total.sim_duration,
        played_duration: total.played_duration,
    };

    if is_fast {
        // Scale up the interval, multiplier and offset by the simulated length.
        interval = CycleInterval::new(
            interval
                .start()
                .mul(total.sim_duration)?,
            interval.end().mul(total.sim_duration)?,
        );
        offset = offset.mul(total.sim_duration)?;
        multiplier = multiplier.mul(total.sim_duration)?;
    }

    Ok(PlayMultiple(make_play_multiple(make_outer_iter(
        interval,
        total_props,
        offset,
        multiplier,
    ))))
}
