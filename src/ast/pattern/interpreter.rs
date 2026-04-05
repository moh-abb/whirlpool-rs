use core::cell::RefCell;
use core::fmt::Debug;
use core::iter;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::ast::pattern::Pattern;
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
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleTimeInterval::new(self.start, self.start + self.duration);
        play_multiple(
            interval,
            CycleTime::from_int(i32::from(multiple.length())),
            || multiple.iter(self.arenas.get_pattern_chain_arena()),
            false,
            |_| CycleTime::ONE,
            self.offset,
            self.multiplier,
            |subpattern, subinterval, suboffset, submultiplier| {
                let mut inner_mut = self.inner.borrow_mut();
                let visitor = InterpreterVisitor {
                    arenas: self.arenas,
                    start: subinterval.start(),
                    duration: subinterval.end() - subinterval.start(),
                    offset: suboffset,
                    multiplier: submultiplier,
                    inner: RefCell::new(VisitorInner {
                        player: inner_mut.player,
                    }),
                };
                visit_pattern(&visitor, subpattern.clone())
            },
        )
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let interval =
            CycleTimeInterval::new(self.start, self.start + self.duration);
        play_multiple(
            interval,
            CycleTime::from_int(i32::from(multiple.length())),
            || multiple.iter(self.arenas.get_pattern_chain_arena()),
            true,
            |_| CycleTime::ONE,
            self.offset,
            self.multiplier,
            |subpattern, subinterval, suboffset, submultiplier| {
                let mut inner_mut = self.inner.borrow_mut();
                let visitor = InterpreterVisitor {
                    arenas: self.arenas,
                    start: subinterval.start(),
                    duration: subinterval.end() - subinterval.start(),
                    offset: suboffset,
                    multiplier: submultiplier,
                    inner: RefCell::new(VisitorInner {
                        player: inner_mut.player,
                    }),
                };
                visit_pattern(&visitor, subpattern.clone())
            },
        )
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
        _multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let unit_duration = self.multiplier.recip();
        play_multiple(
            CycleTimeInterval::new(self.start, self.start + self.duration),
            CycleTime::ONE,
            || iter::once(SoundUnit::new(unit, unit_duration)),
            false,
            |_| CycleTime::ONE,
            self.offset,
            self.multiplier,
            |soundunit, subinterval, suboffset, submultiplier| {
                // Only play if aligned to single cycle
                if subinterval.start() == subinterval.start().floor() {
                    let scaled_start =
                        (subinterval.start() + suboffset) / submultiplier;
                    self.inner
                        .borrow_mut()
                        .player
                        .schedule_note_unit(soundunit.clone(), scaled_start);
                }
            },
        );
    }

    fn map_silence(&self) -> Self::PatternOutput {}
}

fn play_slow_multiple<T: Debug, Iter: Iterator<Item = T>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    length_of_elem: impl Fn(&T) -> CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
    mut play_elem: impl FnMut(&T, CycleTimeInterval, CycleTime, CycleTime),
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
    for elem in elements() {
        // Mathematically, elem_length = t_k
        let elem_length = length_of_elem(&elem);
        let elem_interval =
            CycleTimeInterval::new(elem_start, elem_start + elem_length);

        // Upper and lower bounds of rep.
        // Not all of these will be inside the played interval, so we need to
        // verify its intersection.
        let first_rep =
            ((interval.start() - elem_interval.start()) / length).floor();
        let last_rep = ((interval.end() - elem_interval.end()) / length).ceil();

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

                play_elem(&elem, sim_interval, offset + rep_offset, multiplier);
            }

            rep_start += length;
        }

        elem_start = elem_interval.end();
    }
}

fn play_fast_multiple<T: Debug, Iter: Iterator<Item = T>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    length_of_elem: impl Fn(&T) -> CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(&T, CycleTimeInterval, CycleTime, CycleTime),
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
        length_of_elem,
        scaled_offset,
        scaled_multiplier,
        play_elem,
    )
}

#[inline]
fn play_multiple<T: Debug, Iter: Iterator<Item = T>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    is_fast: bool,
    length_of_elem: impl Fn(&T) -> CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(&T, CycleTimeInterval, CycleTime, CycleTime),
) {
    debug_assert!({
        let unit_length_sum = elements()
            .map(|elem| length_of_elem(&elem))
            .fold(CycleTime::ZERO, CycleTime::add);
        length == unit_length_sum
    });
    let play_func =
        if is_fast { play_fast_multiple } else { play_slow_multiple };
    play_func(
        interval,
        length,
        elements,
        length_of_elem,
        offset,
        multiplier,
        play_elem,
    )
}

#[cfg(test)]
pub fn test_play_multiple<T: Debug, Iter: Iterator<Item = T>>(
    interval: CycleTimeInterval,
    length: CycleTime,
    elements: impl Fn() -> Iter,
    is_fast: bool,
    length_of_elem: impl Fn(&T) -> CycleTime,
    offset: CycleTime,
    multiplier: CycleTime,
    play_elem: impl FnMut(&T, CycleTimeInterval, CycleTime, CycleTime),
) {
    play_multiple(
        interval,
        length,
        elements,
        is_fast,
        length_of_elem,
        offset,
        multiplier,
        play_elem,
    )
}
