use core::cell::RefCell;
use core::fmt::Debug;
use core::iter;
use core::iter::repeat;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::arena::Arena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::ast::time::CycleTime;
use crate::ast::time::CycleTimeInterval;
use crate::player::PatternPlayer;
use crate::player::unit::SoundUnit;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;

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
struct PlayElemArgs<'a, T> {
    elem: &'a T,
    interval: CycleTimeInterval,
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

    fn map_cat_or_seq(&self, multiple: Multiple<Pattern>, is_fast: bool) {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleTimeInterval::new(self.start, self.start + self.duration);
        play_multiple(
            interval,
            CycleTime::from_int(i32::from(multiple.length())),
            || {
                repeat(CycleTime::ONE)
                    .zip(multiple.iter(self.arenas.get_pattern_chain_arena()))
            },
            is_fast,
            self.offset,
            self.multiplier,
            self.play_elem_func(),
        )
    }

    fn map_arrange_or_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
        is_fast: bool,
    ) {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleTimeInterval::new(self.start, self.start + self.duration);
        let elems_with_durations = || {
            multiple
                .iter(self.arenas.get_timed_step_chain_arena())
                .filter_map(|timed_step| {
                    self.arenas
                        .get_timed_step_arena()
                        .inspect(timed_step, Clone::clone)
                        .map(|TimedStep(unit, pattern)| (unit, pattern))
                        .ok()
                })
        };
        // TODO: Store the total length to reduce repeated calculation
        let length = elems_with_durations()
            .map(|(dur, _)| dur)
            .fold(CycleTime::ZERO, CycleTime::add);
        play_multiple(
            interval,
            length,
            elems_with_durations,
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
        self.map_arrange_or_time_cat(multiple, true);
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        self.map_arrange_or_time_cat(multiple, false);
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let unit_duration = self.multiplier.recip();
        play_multiple(
            CycleTimeInterval::new(self.start, self.start + self.duration),
            CycleTime::ONE,
            || {
                iter::once((
                    CycleTime::ONE,
                    SoundUnit::new(unit, unit_duration),
                ))
            },
            false,
            self.offset,
            self.multiplier,
            |args| {
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
#[allow(unused)]
fn lerp_interval(
    x1: CycleTimeInterval,
    x2: CycleTimeInterval,
    y2: CycleTimeInterval,
) -> Option<CycleTimeInterval> {
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
    Some(CycleTimeInterval::new(start, end))
}

#[inline]
fn play_intersection<T: Debug>(
    elem: &T,
    rep_interval: CycleTimeInterval,
    rep_intersection: CycleTimeInterval,
    sim_interval: CycleTimeInterval,
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

fn play_elements<T: Debug, Iter: Iterator<Item = (CycleTime, T)>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    offset: CycleTime,
    multiplier: CycleTime,
    mut play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    // Cat and similar patterns play alternating elements.
    // This is achieved by looking at the "repetition" (i.e., current time
    // divided by cycle length) and offsetting the played subpattern.
    //
    // Say we have a sequence { x1, x2, ..., x_{k + 1}, ..., x_n },
    // with corresponding element lengths { t1, t2, ..., t_{k + 1}, ..., t_n }.
    //
    // Denote S(k) = t1 + t2 + ... + t_k and L = length == S(n).
    //
    // Each unit repeats at different times:
    // x1 : actually played at [0, t1), [T, T + t1), [2*T, 2*T + t1), ...
    // simulated at [0, t1), [t1, 2*t1), [2*t1, 3*t1)
    // i.e. offsets 0, T - t1, 2*(T - t1), ...
    // ...
    // x_{k+1} : actually played at [S(k), S(k+1)), [T + S(k), T + S(k+1)),
    //                              [2*T + S(k), 2*T + S(k+1)), ...,
    //                              [rep*T + S(k), rep*T + S(k+1)]), ...
    // simulated at [0, t_{k+1}), [t_{k+1}, 2*t_{k+1}), ...,
    //              [rep*t1, (rep + 1)*t1), ...
    // i.e. offsets S(k), S(k) + (T - t_{k+1}), S(k) + 2*(T - t_{k+1}), ...,
    //              S(k) + rep*(T - t_{k+1}), ...
    //
    // For example: Cat(Cat(A, B), Cat(C, D)) has length 2.0.
    // With multiplier 1.0, the given intervals map to the subintervals
    // with offsets (and repetitions):
    // [0, 1) -> element #0, interval [0, 1), offset 0 (repetition 0)
    // [1, 2) -> element #1, interval [0, 1), offset 1 (repetition 0)
    // [2, 3) -> element #0, interval [1, 2), offset 1 (repetition 1)
    // [3, 4) -> element #1, interval [1, 2), offset 2 (repetition 1)
    // [4, 5) -> element #0, interval [2, 3), offset 2 (repetition 2)
    // [5, 6) -> element #1, interval [2, 3), offset 3 (repetition 2)
    // and so on.

    // Iterate over k = 1 to n.
    // Mathematically, elem_start = S(k-1) with initially S(k-1) = S(0) = 0.
    let mut elem_start = CycleTime::ZERO;
    for (elem_length, elem) in elements() {
        // Mathematically, elem_length = t_k
        let elem_interval =
            CycleTimeInterval::new(elem_start, elem_start + elem_length);

        // Upper and lower bounds of rep.
        // Not all of these will be inside the played interval, so we need to
        // verify its intersection.
        // Due to rounding errors, we need to conservatively estimate
        // the difference in repetitions by taking the maximum possible distance
        // between the interval and elem interval's endpoints.
        let max_start_difference =
            interval.start().floor() - elem_interval.start().ceil();
        let max_end_difference =
            interval.end().ceil() - elem_interval.end().floor();
        let first_rep = (max_start_difference / length).floor();
        let last_rep = (max_end_difference / length).ceil();

        debug_assert!(
            elem_start + first_rep * length <= interval.start(),
            "first repetition to try should start on or before interval"
        );
        debug_assert!(
            elem_start + last_rep * length + elem_length >= interval.end(),
            "last repetition to try should end on or after interval"
        );

        // INV: rep_start == elem_start + rep * length
        // Mathematically, rep_start = rep * T + S(k-1)
        let mut rep_start = elem_start + first_rep * length;
        for rep in first_rep.to_int()..=last_rep.to_int() {
            debug_assert_eq!(
                rep_start,
                elem_start + CycleTime::from_int(rep) * length
            );

            // Mathematically, rep_interval = [rep * T + S(k-1), rep * T + S(k))
            let rep_interval =
                CycleTimeInterval::new(rep_start, rep_start + elem_length);
            // Mathematically, rep_offset = S(k-1) + rep * (T - t_k)
            let rep_offset =
                CycleTime::from_int(rep) * (length - elem_length) + elem_start;

            if let Some(intersect) = rep_interval.intersection(interval) {
                // Calculate the simulated time from the played time.
                // In future, this should use `elem_start` for efficiency
                // (but we would also need to handle when `rep_interval`
                // is not fully covered by `interval`).

                // If rep_interval is fully inside interval,
                // then intersect == rep_interval
                //                == [rep * T + S(k-1), rep * T + S(k))
                // and so
                // sim_interval == [rep * T + S(k-1) - rep_offset,
                //                  rep * T + S(k) - rep_offset)
                //              == [rep * T + S(k-1) - S(k-1) - rep * (T - t_k),
                //                  rep * T + S(k) - S(k-1) - rep * (T - t_k))
                //              == [rep * T - rep * (T - t_k),
                //                  rep * T - rep * (T - t_k) + t_k)
                //              == [rep + t_k, (rep + 1) * t_k)
                debug_assert_eq!(
                    rep_interval.start() - rep_offset,
                    CycleTime::from_int(rep) * elem_length
                );
                debug_assert_eq!(
                    rep_interval.end() - rep_offset,
                    CycleTime::from_int(rep + 1) * elem_length
                );

                let sim_interval = CycleTimeInterval::new(
                    intersect.start() - rep_offset,
                    intersect.end() - rep_offset,
                );
                // Mathematically, if rep_interval is fully inside interval,
                // then sim_interval.start() == rep * t_k
                if rep_interval.start() >= interval.start() {
                    debug_assert_eq!(
                        sim_interval.start(),
                        rep_interval.start() - rep_offset,
                    );
                }
                // ... and sim_interval.end() == (rep + 1) * t_k
                if rep_interval.end() <= interval.end() {
                    debug_assert_eq!(
                        sim_interval.end(),
                        rep_interval.end() - rep_offset,
                    );
                }

                let sim_offset = offset + rep_offset;
                let args = PlayElemArgs {
                    elem: &elem,
                    interval: sim_interval,
                    offset: sim_offset,
                    multiplier,
                };
                play_elem(args);
            }

            rep_start += length;
        }

        elem_start = elem_interval.end();
    }
}

fn play_slow_multiple<T: Debug, Iter: Iterator<Item = (CycleTime, T)>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    play_elements(interval, length, elements, offset, multiplier, play_elem)
}

fn play_fast_multiple<T: Debug, Iter: Iterator<Item = (CycleTime, T)>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    // The patterns for Seq and similar to Cat. Here we
    // there is no repetition.
    //
    // As with `play_slow_multiple`, say we have the same sequence and element
    // lengths.
    // However, here the element lengths become divided over the total length.
    //
    // For example: Seq(Cat(A, B), Cat(C, D)) has length 2.0.
    // With multiplier 1.0, the given intervals map to the subintervals
    // with offsets (and repetitions):
    // Each subpattern will have multiplier 2.0 (= 1.0 * length)
    // [0/2, 1/2) -> element #0, interval [0, 1), offset 0 (repetition 0)
    //               plays A @ [0/2, 1/2)
    // [1/2, 2/2) -> element #1, interval [0, 1), offset 1 (repetition 0)
    //               plays C @ [1/2, 2/2)
    // [2/2, 3/2) -> element #0, interval [1, 2), offset 1 (repetition 1)
    //               plays B @ [2/2, 3/2)
    // [3/2, 4/2) -> element #1, interval [1, 2), offset 2 (repetition 1)
    //               plays D @ [3/2, 4/2)
    // [4/2, 5/2) -> element #0, interval [2, 3), offset 2 (repetition 2)
    //               plays A @ [4/2, 5/2)
    // [5/2, 6/2) -> element #1, interval [2, 3), offset 3 (repetition 2)
    //               plays C @ [5/2, 6/2)
    // and so on.
    let scaled_interval = CycleTimeInterval::new(
        interval.start() * length,
        interval.end() * length,
    );
    let scaled_offset = offset * length;
    let scaled_multiplier = multiplier * length;
    play_slow_multiple(
        scaled_interval,
        length,
        elements,
        scaled_offset,
        scaled_multiplier,
        play_elem,
    )
}

#[inline]
fn play_multiple<T: Debug, Iter: Iterator<Item = (CycleTime, T)>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(PlayElemArgs<'_, T>),
) {
    debug_assert_ne!(length, CycleTime::ZERO);
    debug_assert!({
        let unit_length_sum = elements()
            .map(|(elem_length, _)| elem_length)
            .fold(CycleTime::ZERO, CycleTime::add);
        length == unit_length_sum
    });
    let play_func =
        if is_fast { play_fast_multiple } else { play_slow_multiple };
    play_func(interval, length, elements, offset, multiplier, play_elem)
}

#[cfg(test)]
pub fn test_play_multiple<T: Debug, Iter: Iterator<Item = (CycleTime, T)>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    is_fast: bool,
    offset: CycleTime,
    multiplier: CycleTime,
    mut play_elem: impl FnMut(&T, CycleTimeInterval, CycleTime, CycleTime),
) {
    play_multiple(
        interval,
        length,
        elements,
        is_fast,
        offset,
        multiplier,
        |args| {
            play_elem(args.elem, args.interval, args.offset, args.multiplier)
        },
    )
}
