use core::cell::RefCell;
use core::fmt::Debug;
use core::iter;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::ast::ElemProps;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::mem::Arena;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::player::PatternPlayer;
use crate::player::unit::SoundUnit;

mod private {
    pub trait Sealed {}
}

pub trait InterpreterBorrowAdapter<T>: private::Sealed {
    fn borrow_mut(&mut self) -> impl DerefMut<Target = T>;
}

impl<T> private::Sealed for &RefCell<T> {}
impl<T> InterpreterBorrowAdapter<T> for &RefCell<T> {
    fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
        RefCell::borrow_mut(self)
    }
}

impl<T> private::Sealed for &mut T {}
impl<T> InterpreterBorrowAdapter<T> for &mut T {
    fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
        self.deref_mut()
    }
}

/// An interpreter of [Pattern]s, which keeps track of a current time and
/// plays units (traversing the [Pattern]'s tree) when new units are
/// encountered.
#[allow(unused)]
pub struct Interpreter<'a, Arenas, Player, B> {
    pattern: Index<Pattern>,
    arenas: &'a Arenas,
    borrow_adapter: B,
    position: CycleTime,
    base_multiplier: CycleTime,
    base_offset: CycleTime,
    phantom: PhantomData<Player>,
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer>
    Interpreter<'a, Arenas, Player, &'a mut Player>
{
    #[allow(unused)]
    pub fn new(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        player: &'a mut Player,
    ) -> Self {
        Self {
            pattern,
            arenas,
            borrow_adapter: player,
            position: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer>
    Interpreter<'a, Arenas, Player, &'a RefCell<Player>>
{
    #[allow(unused)]
    pub fn new_with_refcell(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        player: &'a RefCell<Player>,
    ) -> Self {
        Self {
            pattern,
            arenas,
            borrow_adapter: player,
            position: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas, Player, Borrow> Interpreter<'a, Arenas, Player, Borrow> {
    #[cfg(test)]
    #[allow(unused)]
    pub fn set_multiplier(&mut self, multiplier: CycleTime) {
        self.base_multiplier = multiplier;
    }

    #[cfg(test)]
    #[allow(unused)]
    pub fn set_offset(&mut self, offset: CycleTime) {
        self.base_offset = offset;
    }
}

impl<
    'a,
    Arenas: PatternArenas,
    Player: PatternPlayer,
    Borrow: InterpreterBorrowAdapter<Player>,
> Interpreter<'a, Arenas, Player, Borrow>
{
    #[allow(unused)]
    pub fn update_time(&mut self, next_position: CycleTime) {
        // The time should be monotonically increasing.
        assert!(next_position >= self.position);
        let mut borrowed_player = self.borrow_adapter.borrow_mut();
        let visitor = InterpreterVisitor {
            arenas: self.arenas,
            start: self.position,
            duration: next_position.sub(self.position),
            offset: self.base_offset,
            multiplier: self.base_multiplier,
            inner: RefCell::new(VisitorInner {
                player: borrowed_player.deref_mut(),
            }),
        };
        visit_pattern(&visitor, self.pattern.clone());
        self.position = next_position;
    }
}

/// A visitor used to traverse a given [Pattern].
/// - `arenas` is a reference to the arenas where a given [Pattern] is stored.
/// - `interval` is the cycle time (start inclusive, end exclusive) to play
///   any given units.
/// - `multiplier` is used to scale down the duration of notes; i.e., play
///   units faster.
/// - `offset` is used to add an offset to the start time of units played.
/// - `inner` contains the `Player` from which units will be scheduled.
struct InterpreterVisitor<'a, Arenas, Player> {
    arenas: &'a Arenas,
    start: CycleTime,
    duration: CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
    inner: RefCell<VisitorInner<'a, Player>>,
}

/// Represents the arguments that are passed into the `play_elem` function in
/// [play_elements].
#[derive(Debug)]
struct PlayElemArgs<'a, T> {
    elem: &'a T,
    interval: CycleInterval,
    offset: CycleTime,
    multiplier: CycleTime,
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer>
    InterpreterVisitor<'a, Arenas, Player>
{
    fn play_elem_func(&self) -> impl FnMut(PlayElemArgs<'_, Index<Pattern>>) {
        |args| {
            let mut inner_mut = self.inner.borrow_mut();
            let visitor = InterpreterVisitor {
                arenas: self.arenas,
                start: args.interval.start(),
                duration: args.interval.end() - args.interval.start(),
                offset: args.offset,
                multiplier: args.multiplier,
                inner: RefCell::new(VisitorInner { player: inner_mut.player }),
            };
            visit_pattern(&visitor, args.elem.clone())
        }
    }

    fn timed_step_iter<'b>(
        &'b self,
        multiple: &'b Multiple<TimedStep>,
    ) -> impl Iterator<Item = TimedStep> + 'b {
        multiple
            .iter(self.arenas.get_timed_step_chain_arena())
            .filter_map(move |timed_step| {
                self.arenas
                    .get_timed_step_arena()
                    .map(timed_step, Clone::clone)
                    .ok()
            })
    }

    fn map_cat_or_seq(&self, multiple: Multiple<Pattern>, is_fast: bool) {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleInterval::new(self.start, self.start + self.duration);
        let make_sim_elem = |elem: Index<Pattern>| ElemProps {
            elem,
            sim_duration: CycleTime::ONE,
            played_duration: CycleTime::ONE,
        };
        let length = CycleTime::from_int(i32::from(multiple.length()));
        let get_elements = || {
            multiple
                .iter(self.arenas.get_pattern_chain_arena())
                .map(make_sim_elem)
        };
        play_multiple(
            interval,
            ElemProps {
                elem: get_elements,
                sim_duration: length,
                played_duration: length,
            },
            is_fast,
            self.offset,
            self.multiplier,
            self.play_elem_func(),
        )
    }
}

struct VisitorInner<'a, Player> {
    player: &'a mut Player,
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer> PatternVisitor
    for InterpreterVisitor<'a, Arenas, Player>
{
    type Output = ();
    type PatternOutput = ();

    fn get_arenas(&self) -> &impl PatternArenas {
        self.arenas
    }

    fn map_pattern(
        &self,
        _pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output {
        pattern_output
    }

    fn map_cat(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        self.map_cat_or_seq(multiple, false);
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        self.map_cat_or_seq(multiple, true);
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        // Plays each of the patterns in parallel.

        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        multiple
            .iter(self.arenas.get_pattern_chain_arena())
            .for_each(|pattern_index| visit_pattern(self, pattern_index));
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleInterval::new(self.start, self.start + self.duration);

        let multiple_length = CycleTime::from_int(i32::from(multiple.length()));
        // TODO: Store the total length to reduce repeated calculation
        let total_cycle_length = self
            .timed_step_iter(&multiple)
            .map(|TimedStep(dur, _)| dur)
            .sum();
        // If we have a [TimeCat], then we simulate over each one cycle in the
        // [Multiple] and then scale each element individually by its
        // proportion of the total; e.g. TimeCat([1, "A"], [2, "B"], [3, "C"])
        // will have element lengths 1*3/6, 2*3/6, 3*3/6
        // (which adds to 3, the number of elements).
        let get_scaled_length = |elem_length: CycleTime| {
            (elem_length * multiple_length) / total_cycle_length
        };
        let make_sim_elem = |TimedStep(elem_length, pattern)| ElemProps {
            elem: pattern,
            sim_duration: CycleTime::ONE,
            played_duration: get_scaled_length(elem_length),
        };
        let get_elements = || {
            self.timed_step_iter(&multiple)
                .map(make_sim_elem)
        };
        // Due to fixed point rounding errors, recalculate the total length
        // after calculating the scaled length of each element.
        let played_duration = get_elements()
            .map(|elem| elem.played_duration)
            .sum();
        play_multiple(
            interval,
            ElemProps {
                elem: get_elements,
                sim_duration: multiple_length,
                played_duration: played_duration,
            },
            true,
            self.offset,
            self.multiplier,
            self.play_elem_func(),
        );
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleInterval::new(self.start, self.start + self.duration);

        // TODO: Store the total length to reduce repeated calculation
        let total_cycle_length = self
            .timed_step_iter(&multiple)
            .map(|TimedStep(dur, _)| dur)
            .sum();
        let make_sim_elem = |TimedStep(elem_length, pattern)| ElemProps {
            elem: pattern,
            sim_duration: elem_length,
            played_duration: elem_length,
        };
        // If we have an [Arrange], then we simulate over all the cycles in
        // the pattern and so the played length is `total_cycle_length`.
        let get_elements = || {
            self.timed_step_iter(&multiple)
                .map(make_sim_elem)
        };
        play_multiple(
            interval,
            ElemProps {
                elem: get_elements,
                sim_duration: total_cycle_length,
                played_duration: total_cycle_length,
            },
            false,
            self.offset,
            self.multiplier,
            self.play_elem_func(),
        )
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let unit_duration = self.multiplier.recip();
        let sound_unit = SoundUnit::new(unit, unit_duration);
        let get_elements = || {
            iter::once(ElemProps {
                elem: sound_unit.clone(),
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            })
        };
        play_multiple(
            CycleInterval::new(self.start, self.start + self.duration),
            ElemProps {
                elem: get_elements,
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            },
            false,
            self.offset,
            self.multiplier,
            |args| {
                debug_assert_eq!(args.elem, &sound_unit);
                // Only play if aligned to single cycle
                let start = args.interval.start();
                if start != start.floor() {
                    return;
                }
                let scaled_start = (start + args.offset) / args.multiplier;
                self.inner
                    .borrow_mut()
                    .player
                    .schedule_note_unit(args.elem.clone(), scaled_start);
            },
        );
    }

    fn map_silence(&self) -> Self::PatternOutput {}
}

// Calculates the interval `y1` with each end as a linear interpolation
// along `y2`, by the same fraction that the corresponding end of `x1` is
// along `x2`.
//
// For example, if x1 = [2, 4), x2 = [0, 8), y2 = [2, 6), then y1 = [3, 4).
#[inline]
fn lerp_interval(
    x1: CycleInterval,
    x2: CycleInterval,
    y2: CycleInterval,
) -> Option<CycleInterval> {
    // Let  alpha     = (x1.start - x2.start) / (x2.end - x2.start)
    //      1 - alpha = (x2.end - x1.start) / (x2.end - x2.start)
    //      beta      = (x1.end - x2.start) / (x2.end - x2.start)
    //      1 - beta  = (x2.end - x1.end) / (x2.end - x2.start)
    // Then with lerp(p, x, y) = p * y + (1 - p) * x,
    // y1.start = lerp(alpha, y2.start, y2.end)
    //      = (
    //          x1.start * (y2.end - y2.start)
    //          + (y2.start * x2.end - y2.end * x2.start)
    //        ) / (x2.end - x2.start)
    //      = (x1.start * y2.length + k) / x2.length
    // y1.end = lerp(beta, y2.start, y2.end)
    //      = (
    //          x1.end * (y2.end - y2.start)
    //          + (y2.start * x2.end - y2.end * x2.start)
    //        ) / (x2.end - x2.start)
    //      = (x1.end * y2.length + k) / x2.length
    // where k = y2.start * x2.end - y2.end * x2.start
    //
    // Trivially, if x1 = x2 then y1 = y2 (in which case alpha = 0, beta = 1).
    if x1 == x2 {
        return Some(y2);
    }
    let x2_length = x2.end().sub(x2.start());
    let y2_length = y2.end().sub(y2.start());
    let k = y2
        .start()
        .mul(x2.end())
        .sub(y2.end().mul(x2.start()));
    let endpoint = |point: CycleTime| {
        point
            .mul(y2_length)
            .add(k)
            .div(x2_length)
    };
    let start = endpoint(x1.start()).max(y2.start());
    let end = endpoint(x1.end()).min(y2.end());
    if start >= end {
        return None;
    }
    Some(CycleInterval::new(start, end))
}

#[inline]
fn play_intersection<T: Debug>(
    elem: &T,
    rep_interval: CycleInterval,
    rep_intersection: CycleInterval,
    sim_interval: CycleInterval,
    elem_offset: CycleTime,
    elem_multiplier: CycleTime,
    play_elem: &mut impl FnMut(PlayElemArgs<'_, T>),
) {
    // `rep_intersection` should fit completely inside `rep_interval`.
    debug_assert_eq!(
        rep_intersection.intersection(rep_interval),
        Some(rep_intersection)
    );

    let opt_sim_intersection =
        lerp_interval(rep_intersection, rep_interval, sim_interval);
    let Some(sim_intersection) = opt_sim_intersection else {
        // Due to fixed point arithmetic errors, the intersection is too small
        // to consider.
        return;
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

fn play_elements<T: Debug, Iter: Iterator<Item = ElemProps<T>>>(
    interval: CycleInterval,
    total: ElemProps<Iter>,
    offset: CycleTime,
    total_multiplier: CycleTime,
    mut play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
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
    for cur in total.elem {
        // INV: played_start == P(i - 1)
        // elem_interval = [P(i - 1), P(i))
        let elem_interval = CycleInterval::new(
            played_start,
            played_start + cur.played_duration,
        );

        // Upper and lower bounds of rep.
        // Not all of these will be inside the played interval, so we need to
        // verify its intersection.
        let max_start_difference =
            interval.start().floor() - elem_interval.start().ceil();
        let max_end_difference =
            interval.end().ceil() - elem_interval.end().floor();
        let first_rep = (max_start_difference / total.played_duration).floor();
        let last_rep = (max_end_difference / total.played_duration).ceil();

        debug_assert!(
            played_start + first_rep * total.played_duration
                <= interval.start(),
            "first repetition to try should start on or before interval"
        );
        debug_assert!(
            played_start
                + last_rep * total.played_duration
                + cur.played_duration
                >= interval.end(),
            "last repetition to try should end on or after interval"
        );

        // elem_multiplier = m_s = (s_i / p_i) * m_t
        let multiplier_scale = cur.sim_duration / cur.played_duration;
        let elem_multiplier = multiplier_scale * total_multiplier;

        // INV: rep_start = rep * PL + P(i - 1)
        let mut rep_start = first_rep * total.played_duration + played_start;
        for rep_index in first_rep.to_int()..=last_rep.to_int() {
            let rep = CycleTime::from_int(rep_index);
            // Verify invariant for rep_start
            debug_assert_eq!(
                rep_start,
                rep * total.played_duration + played_start
            );

            // rep_interval = [rep_start, rep_start + p_i)
            let rep_interval =
                CycleInterval::new(rep_start, rep_start + cur.played_duration);
            // Calculation detailed above.
            let rep_offset = (rep_start + offset) * multiplier_scale
                - rep * cur.sim_duration;

            let opt_rep_intersection = rep_interval.intersection(interval);
            if let Some(rep_intersection) = opt_rep_intersection {
                // sim_interval == [r*s_i, (r+1)*s_i)
                let sim_start = rep * cur.sim_duration;
                let sim_interval =
                    CycleInterval::new(sim_start, sim_start + cur.sim_duration);

                play_intersection(
                    &cur.elem,
                    rep_interval,
                    rep_intersection,
                    sim_interval,
                    rep_offset,
                    elem_multiplier,
                    &mut play_elem,
                );
            }

            rep_start += total.played_duration;
        }

        played_start = elem_interval.end();
    }
}

#[inline]
fn play_slow_multiple<T: Debug, Iter: Iterator<Item = ElemProps<T>>>(
    interval: CycleInterval,
    total: ElemProps<Iter>,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    play_elements(interval, total, offset, multiplier, play_elem)
}

#[inline]
fn play_fast_multiple<T: Debug, Iter: Iterator<Item = ElemProps<T>>>(
    interval: CycleInterval,
    total: ElemProps<Iter>,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    // Scale up the interval, multiplier and offset by the simulated length.
    let scaled_interval = CycleInterval::new(
        interval.start() * total.sim_duration,
        interval.end() * total.sim_duration,
    );
    let scaled_offset = offset * total.sim_duration;
    let scaled_multiplier = multiplier * total.sim_duration;
    play_slow_multiple(
        scaled_interval,
        total,
        scaled_offset,
        scaled_multiplier,
        play_elem,
    )
}

#[inline]
fn play_multiple<T: Debug, Iter: Iterator<Item = ElemProps<T>>>(
    interval: CycleInterval,
    total: ElemProps<impl Fn() -> Iter>,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    if cfg!(debug_assertions) {
        let calculated_sim_duration = (total.elem)()
            .map(|multiple_elem| multiple_elem.sim_duration)
            .sum::<CycleTime>();

        let calculated_played_duration = (total.elem)()
            .map(|multiple_elem| multiple_elem.played_duration)
            .sum::<CycleTime>();

        debug_assert_eq!(calculated_sim_duration, total.sim_duration);
        debug_assert_eq!(calculated_played_duration, total.played_duration);
    }

    let total_props = ElemProps {
        elem: (total.elem)(),
        sim_duration: total.sim_duration,
        played_duration: total.played_duration,
    };
    let play_func =
        if is_fast { play_fast_multiple } else { play_slow_multiple };
    play_func(interval, total_props, offset, multiplier, play_elem)
}

#[cfg(test)]
pub fn test_play_multiple<T: Debug, Iter: Iterator<Item = ElemProps<T>>>(
    interval: CycleInterval,
    total: ElemProps<impl Fn() -> Iter>,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    mut play_elem: impl FnMut(&T, CycleInterval, CycleTime, CycleTime),
) {
    play_multiple(interval, total, is_fast, offset, multiplier, |args| {
        play_elem(args.elem, args.interval, args.offset, args.multiplier)
    })
}
