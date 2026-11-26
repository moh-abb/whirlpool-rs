//! Cat and similar patterns play alternating elements.
//! This is achieved by looking at the "repetitions" of each element.
//!
//! Say we have a sequence of elements { x1, x2, ..., x_{k + 1}, ..., x_n },
//! with their played lengths { p1, p2, ..., p_{k + 1}, ..., p_n }, and
//! simulated lengths { s1, s2, ..., s_{k + 1}, ..., s_n }.
//!
//! Normally, the simulated and played lengths are the same, except for
//! TimeCat where the simulated length is 1, while the played length is
//! total played length / (element played length * number of elements).
//!
//! Denote P(k) = p1 + p2 + ... + p_k and PL = total_played_duration == P(n),
//! and    S(k) = s1 + s2 + ... + s_k and SL = total_sim_duration    == S(n).
//!
//! Each unit repeats at different times:
//! x1:        played at [0, p1), [PL, PL + p1), [2*PL, 2*PL + p1), ...
//!         simulated at [0, s1), [s1, 2*s1),    [2*s1, 3*s1), ...
//! x2:        played at [P(1), P(2)), [PL + P(1), PL + P(2)),
//!                      [2*PL + P(1), 2*PL + P(2)), ...
//!         simulated at [0, s2), [s2, 2*s2), [2*s2, 3*s2), ...
//! ...
//! x_{i + 1}: played at [P(i), P(i+1)), [PL + P(i), PL + P(i+1)),
//!                      [2*PL + P(i), 2*PL + P(i+1)), ...
//!         simulated at [0, s_{i+1}), [s_{i+1}, 2*s_{i+1}),
//!                      [2*s_{i+1}, 3*s_{i+1}), ...
//! We say the nth pair of "played/simulated at" is the nth repetition,
//! for r >= 0:
//! x_{i + 1}'s r'th repetition: played at [r*PL + P(i), r*PL + P(i+1))
//!                           simulated at [r*s_{i+1}, (r+1)*s_{i+1})
//! The multiplier of any repetition of x_{i+1} is (s_{i+1} / p_{i+1}) * m_t
//! where m_t is the total multiplier.
//! (i.e., the multiplier is proportional to the ratio between the simulated
//! played lengths).
//!
//! The law that relates played and simulated times is:
//! T_p = (T_s + o) / m_s where T_p = played start time
//!                             T_s = simulated start time
//!                             o   = simulated offset
//!                             m_s = simulated multiplier
//!                                 = (s_i / p_i) * m_t (as earlier).
//! Then for element x_{i+1}, repetition i:
//! T_p = r*PL + P(i)       (by definition of a repetition earlier)
//! T_s = r*s_{i+1}         (ditto)
//! m_s = (s_i / p_i) * m_t (as defined earlier)
//! so r*PL + P(i)   = (r*s_{i+1} + o) / ((s_i / p_i) * m_t)
//!    r*s_{i+1} + o = (r*PL + P(i)) * (s_{i+1} / p_{i+1}) * m_t
//!    o             = (r*PL + P(i)) * (s_{i+1} / p_{i+1}) * m_t - r*s_{i+1}
//!                  = rep_start * elem_multiplier - rep * cur.sim_duration
//!
//! Iterate over i in [1, n].
//! INV: played_start == P(i - 1)

use core::fmt::Debug;
use core::ops::RangeInclusive;

use crate::ast::expr::linked::ExprArenas;
use crate::ast::time::CycleTime;
use crate::ast::time::interval::CycleInterval;
use crate::impl_frame;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::query::QueryMode;
use crate::interpreter::value::ValueStack;

/// Defines the strategy to obtain the duration of a given concatenated
/// element. This is done by either giving a constant value via
/// [TimingSource::Constant], or another frame via [TimingSource::ViaFrame]
/// that, when executed, will place the constant value onto the stack.
pub trait TimingStrategy {
    type Item: Clone + Debug;

    fn next_item(
        &mut self,
        arenas: &impl ExprArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>;

    fn pop_elem_props(
        &mut self,
        stack: &mut impl ValueStack,
    ) -> PatternInterpreterResult<ElemProps<Self::Item>>;

    fn total_props(
        &mut self,
        arenas: &impl ExprArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>>;

    fn eval_item(
        &mut self,
        item: Self::Item,
        intersection: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<InterpreterFrame>;
}

/// Determines how the timing data for the frame will be calculated.
/// See [TimingStrategy].
pub enum TimingSource<T> {
    Const(T),
    ViaFrame(InterpreterFrame),
}

#[inline]
fn play_intersection<T>(
    item: T,
    rep_interval: CycleInterval,
    rep_intersection: CycleInterval,
    sim_interval: CycleInterval,
    elem_offset: CycleTime,
    elem_multiplier: CycleTime,
) -> PatternInterpreterResult<Option<(T, PlayElemArgs<()>)>> {
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

    Ok(Some((
        item,
        PlayElemArgs {
            elem: (),
            interval: sim_intersection,
            offset: elem_offset,
            multiplier: elem_multiplier,
        },
    )))
}

#[derive(derive_new::new, Debug, Clone)]
struct OuterIterData {
    /// Used to calculate intersections of alternating elements.
    /// Not used for each repetition of an element.
    #[new(value = "CycleTime::ZERO")]
    played_start: CycleTime,
    /// Only used to verify the total lengths at the end of iteration.
    #[new(value = "CycleTime::ZERO")]
    sim_start: CycleTime,
    interval: CycleInterval,
    total: ElemProps<()>,
    offset: CycleTime,
    total_multiplier: CycleTime,
}

impl OuterIterData {
    fn update<T>(
        &mut self,
        cur: ElemProps<T>,
    ) -> PatternInterpreterResult<(RepIterData<T>, RangeInclusive<i32>)> {
        // INV: played_start == P(i - 1)
        // elem_interval = [P(i - 1), P(i))
        let elem_interval = CycleInterval::new(
            self.played_start,
            self.played_start
                .add(cur.played_duration)?,
        );

        // Upper and lower bounds of rep.
        // Not all of these will be inside the played interval,
        // so we need to verify its intersection.
        let max_start_difference = self
            .interval
            .start()
            .floor()?
            .sub(elem_interval.start().ceil()?)?;
        let max_end_difference = self
            .interval
            .end()
            .ceil()?
            .sub(elem_interval.end().floor()?)?;
        let first_rep = max_start_difference
            .div(self.total.played_duration)?
            .floor()?;
        let last_rep = max_end_difference
            .div(self.total.played_duration)?
            .ceil()?;

        debug_assert!(
            self.played_start
                .add(first_rep.mul(self.total.played_duration)?)?
                <= self.interval.start(),
            "first repetition to try should start on or before interval"
        );
        debug_assert!(
            self.played_start
                .add(last_rep.mul(self.total.played_duration)?)?
                .add(cur.played_duration)?
                >= self.interval.end(),
            "last repetition to try should end on or after interval"
        );

        // elem_multiplier = m_s = (s_i / p_i) * m_t
        let multiplier_scale = cur
            .sim_duration
            .div(cur.played_duration)?;
        let elem_multiplier = multiplier_scale.mul(self.total_multiplier)?;

        // INV: rep_start = rep * PL + P(i - 1)
        let rep_start = first_rep
            .mul(self.total.played_duration)?
            .add(self.played_start)?;

        // Note that we need to use the older value of the played start
        // when processing all the repetitions using the old value.
        self.played_start = elem_interval.end();
        self.sim_start = self.sim_start.add(cur.sim_duration)?;

        Ok((
            RepIterData {
                rep_start,
                played_start: elem_interval.start(),
                cur,
                multiplier_scale,
                elem_multiplier,
            },
            first_rep.to_int()..=last_rep.to_int(),
        ))
    }
}

#[derive(Debug, Clone)]
struct RepIterData<T> {
    /// Played start from the outer scope.
    /// Used for each repetition of an element.
    played_start: CycleTime,
    rep_start: CycleTime,
    cur: ElemProps<T>,
    multiplier_scale: CycleTime,
    elem_multiplier: CycleTime,
}

impl<T: Clone> RepIterData<T> {
    fn update(
        &mut self,
        rep_index: i32,
        outer: &OuterIterData,
    ) -> PatternInterpreterResult<Option<(T, PlayElemArgs<()>)>> {
        // Because `RepIterData` was called after the played start has been
        // updated, we need to subtract it again.
        let rep = CycleTime::checked_from_int(rep_index)?;
        // Verify invariant for rep_start
        // Important: We need to use the old value of `played_start` stored
        // in the `RepIterData` instead of the updated value in `outer`.
        debug_assert_eq!(
            self.rep_start,
            rep.mul(outer.total.played_duration)?
                .add(self.played_start)?
        );

        // rep_interval = [rep_start, rep_start + p_i)
        let rep_interval = CycleInterval::new(
            self.rep_start,
            self.rep_start
                .add(self.cur.played_duration)?,
        );
        // Calculation detailed above.
        let rep_offset = self
            .rep_start
            .add(outer.offset)?
            .mul(self.multiplier_scale)?
            .sub(rep.mul(self.cur.sim_duration)?)?;

        let opt_rep_intersection = rep_interval.intersection(outer.interval);

        self.rep_start = self
            .rep_start
            .add(outer.total.played_duration)?;

        let Some(rep_intersection) = opt_rep_intersection else {
            return Ok(None);
        };

        // sim_interval == [r*s_i, (r+1)*s_i)
        let sim_start = rep.mul(self.cur.sim_duration)?;
        let sim_interval = CycleInterval::new(
            sim_start,
            sim_start.add(self.cur.sim_duration)?,
        );

        let opt_intersection = play_intersection(
            self.cur.elem.clone(),
            rep_interval,
            rep_intersection,
            sim_interval,
            rep_offset,
            self.elem_multiplier,
        )?;

        Ok(opt_intersection)
    }
}

impl_frame!(
    #[derive(derive_new::new, derive_more::Debug, Clone)]
    pub struct PlayMultiple {
        #[new(value = "State::start {}")]
        state: State<Timing::Item>,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
        timing: Timing,
        is_fast: bool,
    },
    [Timing] = [Timing = TimingStrategy] + [Timing::Item = Clone],
    #[derive(derive_more::Debug, Clone)]
    enum State {},
    [Elem] = [],
    [self, value_stack, arenas],
    /// The start state.
    fn start() {
        match self.timing.total_props(arenas)? {
            TimingSource::Const(total_props) => {
                let outer_data = total_props_to_outer_data(
                    total_props,
                    &mut self.play_args,
                    self.is_fast,
                )?;
                (processing_begin { outer_data }, None)
            }
            TimingSource::ViaFrame(frame) => {
                (pending_total_props {}, Some(frame))
            }
        }
    },
    /// Waiting to get the total elements' sim and played properties via a
    /// frame, which will push these properties onto the value stack.
    fn pending_total_props() {
        let total_props = value_stack.pop()?;
        let outer_data = total_props_to_outer_data(
            total_props,
            &mut self.play_args,
            self.is_fast,
        )?;
        (processing_begin { outer_data }, None)
    },
    /// Obtained total props but pending data for the next element to be
    /// calculated.
    fn processing_begin(outer_data: OuterIterData) {
        match self.timing.next_item(arenas)? {
            Some(TimingSource::Const(cur)) => {
                elem_props_to_state::<Timing>(outer_data, cur)?
            }
            Some(TimingSource::ViaFrame(frame)) => {
                (pending_elem { outer_data }, Some(frame))
            }
            None => (finish_outer { outer_data }, None),
        }
    },
    /// Obtained total props but pending the next element itself to be
    /// calculated.
    fn pending_elem(outer_data: OuterIterData) {
        let cur = self
            .timing
            .pop_elem_props(value_stack)?;
        elem_props_to_state::<Timing>(outer_data, cur)?
    },
    /// Currently processing some repetitions of one specific element.
    fn processing_current(
        outer_data: OuterIterData,
        inner_data: RepIterData<Elem>,
        indices: RangeInclusive<i32>,
    ) {
        let mut mut_inner_data = inner_data;
        let mut mut_indices = indices;
        match mut_indices.next() {
            Some(rep_index) => {
                let opt_frame = if let Some((item, intersection)) =
                    mut_inner_data.update(rep_index, &outer_data)?
                {
                    Some(self.timing.eval_item(
                        item,
                        intersection,
                        self.query_mode.clone(),
                    )?)
                } else {
                    None
                };
                (
                    processing_current {
                        outer_data,
                        inner_data: mut_inner_data,
                        indices: mut_indices,
                    },
                    opt_frame,
                )
            }
            None => (processing_begin { outer_data }, None),
        }
    },
    /// No elements left to process and we are done.
    fn finish_outer(outer_data: OuterIterData) {
        check_elem_lengths(
            outer_data.sim_start,
            outer_data.played_start,
            outer_data.total.sim_duration,
            outer_data.total.played_duration,
        )?;
        (finish, None)
    }
);

fn total_props_to_outer_data(
    total_props: ElemProps<()>,
    play_args: &mut PlayElemArgs<()>,
    is_fast: bool,
) -> PatternInterpreterResult<OuterIterData> {
    let PlayElemArgs { elem: (), interval, offset, multiplier } = play_args;

    if is_fast {
        // Scale up the interval, multiplier and offset by the simulated length.
        *interval = CycleInterval::new(
            interval
                .start()
                .mul(total_props.sim_duration)?,
            interval
                .end()
                .mul(total_props.sim_duration)?,
        );
        *offset = offset.mul(total_props.sim_duration)?;
        *multiplier = multiplier.mul(total_props.sim_duration)?;
    }

    PatternInterpreterResult::Ok(OuterIterData::new(
        play_args.interval,
        total_props,
        play_args.offset,
        play_args.multiplier,
    ))
}

fn elem_props_to_state<Timing: TimingStrategy>(
    mut outer_data: OuterIterData,
    cur: ElemProps<Timing::Item>,
) -> PatternInterpreterResult<(State<Timing::Item>, Option<InterpreterFrame>)> {
    let (inner_data, indices) = outer_data.update(cur)?;
    PatternInterpreterResult::Ok((
        State::processing_current { outer_data, inner_data, indices },
        Option::<InterpreterFrame>::None,
    ))
}

/// Verifies that the elements of an iterator have the correct simulation
/// properties (simulated and played durations) as expected.
#[inline]
#[must_use]
fn check_elem_lengths(
    calculated_sim_duration: CycleTime,
    calculated_played_duration: CycleTime,
    provided_sim_duration: CycleTime,
    provided_played_duration: CycleTime,
) -> PatternInterpreterResult<()> {
    if cfg!(debug_assertions) {
        debug_assert_eq!(calculated_sim_duration, provided_sim_duration);

        // Leave a little tolerance for `TimeCat`'s played durations.
        let abs_diff = |x: CycleTime, y: CycleTime| {
            if x <= y { y - x } else { x - y }
        };
        let tolerance =
            CycleTime::EPSILON.mul(CycleTime::checked_from_int(2)?)?;
        debug_assert!(
            abs_diff(calculated_played_duration, provided_played_duration)
                <= tolerance,
            "{calculated_played_duration} and {provided_played_duration} \
            differ by more than {tolerance}"
        );
    }

    Ok(())
}
