use core::cell::RefCell;
use core::iter;
use core::iter::repeat_with;
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

/// Splits up a given time (denoted by start time and duration) by a given
/// cycle duration and increment. Returns intervals of duration less than or
/// equal to the increment, with the first returned interval aligning to the
/// cycle length. If one returned interval is not included in the total
/// interval, then the interval is [Option::None].
///
/// If the start (or end) time is not aligned to the increment, then the
/// first (or last) interval will have a duration less than the cycle duration.
fn interval_chunks_with_increments(
    interval: CycleTimeInterval,
    cycle_length: CycleTime,
    increment: CycleTime,
) -> impl Iterator<Item = Option<CycleTimeInterval>> {
    let start = interval.start();
    let end = interval.end();
    let aligned_start = start.round_down_to_nearest(cycle_length);
    let mut window_start = aligned_start;
    let windows = iter::repeat_with(move || {
        let window_end = window_start.add(increment);
        let window = CycleTimeInterval::new(window_start, window_end);
        window_start = window_end;
        window
    });
    windows
        .take_while(move |window| window.start() < end)
        .map(move |window| window.intersection(interval))
}

/// Splits up a given time (denoted by start time and duration) by a given
/// cycle duration, into separate intervals, each with a start time and
/// duration, such that each interval has length less than or equal to the
/// cycle duration.
/// If the start (or end) time is not aligned to the cycle duration, then the
/// first (or last) interval will have a duration less than the cycle duration.
fn interval_chunks(
    interval: CycleTimeInterval,
    cycle_length: CycleTime,
) -> impl Iterator<Item = CycleTimeInterval> {
    interval_chunks_with_increments(interval, cycle_length, cycle_length)
        .flatten()
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
        // Here, `Cat` indicates that the subpatterns inside are played
        // sequentially, so one cycle per subpattern.
        // Let N = multiple.length()
        // The first repetition of `Cat` will start at 0 and end at
        // CycleTime(N / multiplier).
        // The second repetition of `Cat` will start at the above end
        // and end at twice that amount, and so on.
        // Ergo, the repetition's start times occur at multiples of
        // CycleTime(N / multiplier).

        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let cycle_length = CycleTime::from_int(i32::from(multiple.length()));

        // Use the repetition count to calculate subpatterns' offsets.
        let play_subpattern =
            |unit_offset: CycleTime,
             (opt_unit, pattern): (Option<CycleTimeInterval>, _)| {
                if let Some(unit_interval) = opt_unit {
                    let mut inner_mut = self.inner.borrow_mut();
                    let subpattern_visitor = InterpreterVisitor {
                        arenas: self.arenas,
                        start: unit_interval.start().add(unit_offset),
                        duration: CycleTime::ONE,
                        offset: self.offset.sub(unit_offset),
                        multiplier: self.multiplier,
                        inner: RefCell::new(VisitorInner {
                            player: inner_mut.player,
                        }),
                    };
                    visit_pattern(&subpattern_visitor, pattern);
                }
                unit_offset.add(CycleTime::ONE)
            };

        // Find out which repetition we are currently on; i.e., how many cycle
        // lengths have elapsed until the start.
        let start_repetition_count = self.start.div_euclid(cycle_length);
        let play_repetition =
            |repetition_count: CycleTime, repetition: CycleTimeInterval| {
                let units = interval_chunks_with_increments(
                    repetition,
                    cycle_length,
                    CycleTime::ONE,
                );
                units
                    .zip(multiple.iter(self.arenas.get_pattern_chain_arena()))
                    .fold(repetition_count, play_subpattern);
                repetition_count.add(CycleTime::ONE)
            };

        let interval =
            CycleTimeInterval::new(self.start, self.start.add(self.duration));
        interval_chunks(interval, cycle_length)
            .fold(start_repetition_count, play_repetition);
    }

    fn map_seq(&self, _multiple: Multiple<Pattern>) -> Self::PatternOutput {
        todo!()
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        // Plays each of the patterns in parallel.

        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let chain_arena = self.arenas.get_pattern_chain_arena();
        multiple.fold_left(chain_arena, (), |(), pattern_index| {
            visit_pattern(self, pattern_index)
        })
    }

    fn map_time_cat(
        &self,
        _multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let unit_duration = self.multiplier.recip();
        let unit = SoundUnit::new(unit, unit_duration);

        let start = self.start;
        let end = start.add(self.duration);
        let interval = CycleTimeInterval::new(start, end);

        let units_in_cycle = |_repetitions: i32, cycle: CycleTimeInterval| {
            let scaled_start = cycle
                .start()
                .mul(unit_duration)
                .add(self.offset);
            // Only play notes which are aligned to the cycle (i.e., discard
            // windows of size <1 which start midway through a unit).
            let scaled_unit = (cycle.start() == cycle.start().floor())
                .then_some((unit.clone(), scaled_start));
            scaled_unit.into_iter()
        };

        let mut inner = self.inner.borrow_mut();
        let player = &mut inner.player;
        let play_cycle_unit = |(unit, scaled_start): (SoundUnit, CycleTime)| {
            player.schedule_note_unit(unit, scaled_start)
        };

        play_repetitions(interval, 1, units_in_cycle, play_cycle_unit)
    }

    fn map_silence(&self) -> Self::PatternOutput {}
}

fn play_repetitions<CycleUnit, CycleIterator: Iterator<Item = CycleUnit>>(
    interval: CycleTimeInterval,
    cycle_length: i32,
    mut units_in_cycle: impl FnMut(i32, CycleTimeInterval) -> CycleIterator,
    play_cycle_unit: impl FnMut(CycleUnit),
) {
    // Find out which repetition we are currently on; i.e., how many cycle
    // lengths have elapsed until the start.
    let start_repetition_count = interval
        .start()
        .div_euclid(CycleTime::from_int(cycle_length));

    let mut cur_repetition_count = start_repetition_count;
    let repetition_counts = repeat_with(|| {
        let result = cur_repetition_count;
        cur_repetition_count = cur_repetition_count.add(CycleTime::ONE);
        result
    });

    repetition_counts
        .zip(interval_chunks(interval, CycleTime::from_int(cycle_length)))
        .flat_map(|(repetition_count, whole_cycle)| {
            units_in_cycle(repetition_count.to_int(), whole_cycle)
        })
        .for_each(play_cycle_unit);
}
