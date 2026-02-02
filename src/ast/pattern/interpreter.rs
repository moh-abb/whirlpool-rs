use core::cell::RefCell;
use core::iter;
use core::marker::PhantomData;
use core::ops::DerefMut;

use fixed::Wrapping;

use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::ast::time::CycleTime;
use crate::ast::time::CycleTimeInterval;
use crate::player::PatternPlayer;
use crate::player::unit::SoundUnit;

mod private {
    use core::cell::RefCell;
    use core::ops::DerefMut;

    pub trait BorrowAdapter<T> {
        fn borrow_mut(&mut self) -> impl DerefMut<Target = T>;
    }

    impl<T> BorrowAdapter<T> for &RefCell<T> {
        fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
            RefCell::borrow_mut(self)
        }
    }

    impl<T> BorrowAdapter<T> for &mut T {
        fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
            self.deref_mut()
        }
    }
}

#[allow(unused)]
pub struct Interpreter<'a, Arenas, Player, B: private::BorrowAdapter<Player>> {
    pattern: Index<Pattern>,
    arenas: &'a Arenas,
    borrow_adapter: B,
    position: CycleTime,
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
            phantom: PhantomData,
        }
    }
}

impl<
    'a,
    Arenas: PatternArenas,
    Player: PatternPlayer,
    Borrow: private::BorrowAdapter<Player>,
> Interpreter<'a, Arenas, Player, Borrow>
{
    #[allow(unused)]
    pub fn update_time(&mut self, next_position: CycleTime) {
        // The time should be monotonically increasing.
        assert!(next_position >= self.position);
        let mut borrowed_player = self.borrow_adapter.borrow_mut();
        let visitor = InterpreterVisitor {
            arenas: self.arenas,
            interval: CycleTimeInterval::new(self.position, next_position),
            offset: CycleTime::ZERO,
            multiplier: 1,
            inner: RefCell::new(VisitorInner {
                player: borrowed_player.deref_mut(),
            }),
        };
        visit_pattern(&visitor, self.pattern.clone());
        self.position = next_position;
    }
}

struct InterpreterVisitor<'a, Arenas, Player> {
    arenas: &'a Arenas,
    interval: CycleTimeInterval,
    multiplier: u32,
    offset: CycleTime,
    inner: RefCell<VisitorInner<'a, Player>>,
}

trait PatternConsumer<T> {
    fn consume(&self, f: impl FnMut(T, CycleTime));
}

struct CatConsumer<'a, P>(Multiple<Pattern>, &'a P);

impl<P: PatternArenas> PatternConsumer<Index<Pattern>> for CatConsumer<'_, P> {
    fn consume(&self, mut f: impl FnMut(Index<Pattern>, CycleTime)) {
        let Self(multiple, arenas) = self;
        let chain_arena = arenas.get_pattern_chain_arena();
        multiple.fold_left(chain_arena, (), |(), pattern_index| {
            f(pattern_index, CycleTime::ONE)
        })
    }
}

struct NoteConsumer(NoteUnit);

impl PatternConsumer<NoteUnit> for NoteConsumer {
    fn consume(&self, mut f: impl FnMut(NoteUnit, CycleTime)) {
        f(self.0, CycleTime::ONE)
    }
}

/// Plays patterns in `interval` from a given `consumer`:
/// The consumer takes a function that processes subpatterns with their length,
/// and applies it to any patterns stored inside the subpattern.
///
/// The cycle length is determined by the total length (integer cycles) of the
/// consumer. For a singleton [Pattern::Note] or [Pattern::Seq], this is one,
/// for [Pattern::Cat], this is the [Multiple]'s length, and so on.
fn play_consumer<T, F: FnMut(T, CycleTimeInterval, CycleTime)>(
    consumer: impl PatternConsumer<T>,
    interval: CycleTimeInterval,
    cycle_length: u32,
    mut play_subpattern: F,
) {
    // We assume the repetition occurs at multiples of CycleTime(cycle_length).
    let fixed_cycle_length = Wrapping::from_num(cycle_length);

    // Because we may currently be partway through a cycle, we need to start
    // at the beginning of the cycle to be sure.
    let mut init_cycle_start = interval.start().0;
    let time_from_start_of_cycle =
        init_cycle_start.rem_euclid_int(cycle_length);
    init_cycle_start -= time_from_start_of_cycle;
    let init_multiple_repetitions = init_cycle_start
        .div_euclid_int(cycle_length)
        .to_num::<usize>();

    let multiple_starts_and_ends = iter::repeat(())
        .scan(CycleTime(init_cycle_start), |state, ()| {
            let cur_start = *state;
            *state = CycleTime(cur_start.0 + fixed_cycle_length);
            Some(cur_start)
        })
        .take_while(|start| start < &interval.end())
        .enumerate()
        .map(|(i, x)| (i + init_multiple_repetitions, x));

    for (multiple_repetitions, multiple_start) in multiple_starts_and_ends {
        // Choose the pattern(s) to play by performing a modulo on the
        // length of the pattern.
        // If cur_repetition_int = k, then this means that the subpatterns
        // will be played at times k to k + 1.

        // Play each element inside the repetition,
        // ensuring that there is an intersection between the supposed
        // scheduled start/end time, and the visitor's start/end time.
        // times of the
        let mut scheduled_start = multiple_start;
        let visiting_start = Wrapping::from_num(multiple_repetitions);

        // For each item (such as a subpattern or note unit),
        // play it from `visiting_start` to `visiting_start + duration`,
        // such that the scheduled note units occur from `scheduled_start` to
        // `scheduled_start + duration`.
        let fold_item = |item: T, length: CycleTime| {
            // Check that the subpattern is within the interval required.

            // We have the interesting case that the subpattern being played
            // will not always follow the same time as the outer pattern:
            // We want to use the `visiting_start` to obtain
            // the unit notes, but the `multiple_start` to schedule them.
            // Hence, we need to use an offset variable, for which the
            // callee will use to reconstuct the scheduled time.
            let visiting_end = visiting_start + length.0;
            let visiting_interval = CycleTimeInterval::new(
                CycleTime(visiting_start),
                CycleTime(visiting_end),
            );
            let scheduled_end = CycleTime(scheduled_start.0 + length.0);
            let scheduled_interval =
                CycleTimeInterval::new(scheduled_start, scheduled_end);

            let should_play_subpattern = scheduled_interval
                .intersection(interval)
                .is_some();

            if should_play_subpattern {
                // Calculate the times for the subpatterns
                let subpattern_offset = scheduled_start.0 - visiting_start;
                play_subpattern(
                    item,
                    visiting_interval,
                    CycleTime(subpattern_offset),
                );
            }

            // Update the start times for the next subpattern.
            scheduled_start = scheduled_end;
        };

        consumer.consume(fold_item);
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

        let cycle_length = u32::from(multiple.length());
        let play_subpattern =
            |subpattern, visiting_interval, subpattern_offset: CycleTime| {
                let mut inner_mut = self.inner.borrow_mut();
                let subpattern_visitor = InterpreterVisitor {
                    arenas: self.arenas,
                    interval: visiting_interval,
                    offset: CycleTime(self.offset.0 + subpattern_offset.0),
                    multiplier: self.multiplier,
                    inner: RefCell::new(VisitorInner {
                        player: inner_mut.player,
                    }),
                };
                visit_pattern(&subpattern_visitor, subpattern);
            };
        play_consumer(
            CatConsumer(multiple, self.arenas),
            self.interval,
            cycle_length,
            play_subpattern,
        );
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
        let fixed_multiplier = Wrapping::from_num(self.multiplier);
        let unit_duration = CycleTime(fixed_multiplier.recip());
        let sound_unit = SoundUnit::new(unit, unit_duration);

        let play_subpattern =
            |_unit, visiting_interval: CycleTimeInterval, offset: CycleTime| {
                let mut inner = self.inner.borrow_mut();
                let player = &mut inner.player;

                debug_assert_eq!(_unit, unit);
                let to_num = |cycle_time: CycleTime| {
                    cycle_time.0.ceil().to_num::<u32>() / self.multiplier
                };
                let start = to_num(visiting_interval.start());
                let end = to_num(visiting_interval.end());

                for int_time in start..end {
                    let fixed_time = Wrapping::from_num(int_time);
                    let scheduled_start_time =
                        CycleTime(fixed_time + self.offset.0 + offset.0);

                    player.schedule_note_unit(
                        sound_unit.clone(),
                        scheduled_start_time,
                    );
                }
            };
        play_consumer(NoteConsumer(unit), self.interval, 1, play_subpattern);
    }

    fn map_silence(&self) -> Self::PatternOutput {}
}
