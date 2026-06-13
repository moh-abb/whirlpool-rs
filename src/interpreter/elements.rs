use core::fmt::Debug;

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::timed_step_iter;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Multiple;

#[inline]
fn play_intersection<T: Debug>(
    elem: &T,
    rep_interval: CycleInterval,
    rep_intersection: CycleInterval,
    sim_interval: CycleInterval,
    elem_offset: CycleTime,
    elem_multiplier: CycleTime,
    play_elem: &mut impl FnMut(
        PlayElemArgs<'_, T>,
    ) -> Result<(), PatternInterpreterError>,
) -> Result<(), PatternInterpreterError> {
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
        return Ok(());
    };

    // `sim_intersection` should fit completely inside `sim_interval`.
    debug_assert_eq!(
        sim_intersection.intersection(sim_interval),
        Some(sim_intersection),
    );

    let args = PlayElemArgs {
        elem,
        interval: sim_intersection,
        offset: elem_offset,
        multiplier: elem_multiplier,
    };
    play_elem(args)
}

fn play_repetitions<
    T: Debug,
    Iter: Iterator<Item = Result<ElemProps<T>, PatternInterpreterError>>,
>(
    interval: CycleInterval,
    total: ElemProps<Iter>,
    offset: CycleTime,
    total_multiplier: CycleTime,
    mut play_elem: impl FnMut(
        PlayElemArgs<'_, T>,
    ) -> Result<(), PatternInterpreterError>,
) -> Result<(), PatternInterpreterError> {
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
    let mut played_start = CycleTime::ZERO;
    for opt_cur in total.elem {
        let cur = opt_cur?;
        // INV: played_start == P(i - 1)
        // elem_interval = [P(i - 1), P(i))
        let elem_interval = CycleInterval::new(
            played_start,
            played_start.add(cur.played_duration)?,
        );

        // Upper and lower bounds of rep.
        // Not all of these will be inside the played interval, so we need to
        // verify its intersection.
        let max_start_difference = interval
            .start()
            .floor()?
            .sub(elem_interval.start().ceil()?)?;
        let max_end_difference = interval
            .end()
            .ceil()?
            .sub(elem_interval.end().floor()?)?;
        let first_rep = max_start_difference
            .div(total.played_duration)?
            .floor()?;
        let last_rep = max_end_difference
            .div(total.played_duration)?
            .ceil()?;

        debug_assert!(
            played_start.add(first_rep.mul(total.played_duration)?)?
                <= interval.start(),
            "first repetition to try should start on or before interval"
        );
        debug_assert!(
            played_start
                .add(last_rep.mul(total.played_duration)?)?
                .add(cur.played_duration)?
                >= interval.end(),
            "last repetition to try should end on or after interval"
        );

        // elem_multiplier = m_s = (s_i / p_i) * m_t
        let multiplier_scale = cur
            .sim_duration
            .div(cur.played_duration)?;
        let elem_multiplier = multiplier_scale.mul(total_multiplier)?;

        // INV: rep_start = rep * PL + P(i - 1)
        let mut rep_start = first_rep
            .mul(total.played_duration)?
            .add(played_start)?;
        for rep_index in first_rep.to_int()..=last_rep.to_int() {
            let rep = CycleTime::checked_from_int(rep_index)?;
            // Verify invariant for rep_start
            debug_assert_eq!(
                rep_start,
                rep.mul(total.played_duration)?
                    .add(played_start)?
            );

            // rep_interval = [rep_start, rep_start + p_i)
            let rep_interval = CycleInterval::new(
                rep_start,
                rep_start.add(cur.played_duration)?,
            );
            // Calculation detailed above.
            let rep_offset = rep_start
                .add(offset)?
                .mul(multiplier_scale)?
                .sub(rep.mul(cur.sim_duration)?)?;

            let opt_rep_intersection = rep_interval.intersection(interval);
            if let Some(rep_intersection) = opt_rep_intersection {
                // sim_interval == [r*s_i, (r+1)*s_i)
                let sim_start = rep.mul(cur.sim_duration)?;
                let sim_interval = CycleInterval::new(
                    sim_start,
                    sim_start.add(cur.sim_duration)?,
                );

                play_intersection(
                    &cur.elem,
                    rep_interval,
                    rep_intersection,
                    sim_interval,
                    rep_offset,
                    elem_multiplier,
                    &mut play_elem,
                )?
            }

            rep_start = rep_start.add(total.played_duration)?;
        }

        played_start = elem_interval.end();
    }

    Ok(())
}

#[inline]
pub fn play_multiple<
    T: Debug,
    Iter: Iterator<Item = Result<ElemProps<T>, PatternInterpreterError>>,
>(
    mut interval: CycleInterval,
    total: ElemProps<impl Fn() -> Iter>,
    is_fast: bool,
    mut offset: CycleTime,
    mut multiplier: CycleTime,
    play_elem: impl FnMut(
        PlayElemArgs<'_, T>,
    ) -> Result<(), PatternInterpreterError>,
) -> Result<(), PatternInterpreterError> {
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
    play_repetitions(interval, total_props, offset, multiplier, play_elem)
}

pub fn sum_cycle_length<T>(
    mut iter: impl Iterator<Item = Result<T, PatternInterpreterError>>,
    mut f: impl FnMut(T) -> CycleTime,
) -> Result<CycleTime, PatternInterpreterError> {
    iter.try_fold(CycleTime::ZERO, |acc, x| {
        let time = f(x?);
        Ok(acc.add(time)?)
    })
}

pub fn timed_step_total_cycle_length(
    multiple: &Multiple<TimedStep>,
    arenas: &impl PatternArenas,
) -> Result<CycleTime, PatternInterpreterError> {
    sum_cycle_length(
        timed_step_iter(arenas, &multiple)
            .map(|x| x.map_err(PatternInterpreterError::ArenaErr)),
        |x| x.0,
    )
}
