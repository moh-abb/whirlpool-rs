use core::cell::RefCell;
use core::iter;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::ast::time::OverflowError;
use crate::interpreter::Interpreter;
use crate::interpreter::borrow::BorrowAdapter;
use crate::interpreter::elements::play_multiple;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::synth::scheduler::UnitScheduler;
use crate::synth::unit::SoundUnit;

/// An interpreter of [Pattern]s, which keeps track of a current time and
/// plays units (traversing the [Pattern]'s tree) when new units are
/// encountered.
pub struct PatternInterpreter<'a, Arenas, Scheduler, B> {
    pattern: Index<Pattern>,
    arenas: &'a Arenas,
    borrow_adapter: B,
    cur_time: CycleTime,
    base_multiplier: CycleTime,
    base_offset: CycleTime,
    phantom: PhantomData<Scheduler>,
}

impl<'a, Arenas: PatternArenas, Scheduler: UnitScheduler>
    PatternInterpreter<'a, Arenas, Scheduler, &'a mut Scheduler>
{
    #[allow(unused)]
    pub fn new(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        scheduler: &'a mut Scheduler,
    ) -> Self {
        Self {
            pattern,
            arenas,
            borrow_adapter: scheduler,
            cur_time: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas: PatternArenas, Scheduler: UnitScheduler>
    PatternInterpreter<'a, Arenas, Scheduler, &'a RefCell<Scheduler>>
{
    #[allow(unused)]
    pub fn new_with_refcell(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        scheduler: &'a RefCell<Scheduler>,
    ) -> Self {
        Self {
            pattern,
            arenas,
            borrow_adapter: scheduler,
            cur_time: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas, S, Borrow> PatternInterpreter<'a, Arenas, S, Borrow> {
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

impl<'a, Arenas: PatternArenas, S: UnitScheduler, Borrow: BorrowAdapter<S>>
    Interpreter for PatternInterpreter<'a, Arenas, S, Borrow>
{
    type Output = Result<(), OverflowError>;

    #[allow(unused)]
    #[must_use]
    fn update_time(&mut self, next_time: CycleTime) -> Self::Output {
        // The time should be monotonically increasing.
        debug_assert!(next_time >= self.cur_time);
        let mut borrowed_scheduler = self.borrow_adapter.borrow_mut();
        let visitor = InterpreterVisitor {
            arenas: self.arenas,
            interval: CycleInterval::new(self.cur_time, next_time),
            offset: self.base_offset,
            multiplier: self.base_multiplier,
            inner: RefCell::new(VisitorInner {
                scheduler: borrowed_scheduler.deref_mut(),
            }),
        };
        visit_pattern(&visitor, self.pattern.clone());
        self.cur_time = next_time;
        Ok(())
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
struct InterpreterVisitor<'a, Arenas, Scheduler> {
    arenas: &'a Arenas,
    interval: CycleInterval,
    offset: CycleTime,
    multiplier: CycleTime,
    inner: RefCell<VisitorInner<'a, Scheduler>>,
}

impl<'a, Arenas: PatternArenas, Scheduler: UnitScheduler>
    InterpreterVisitor<'a, Arenas, Scheduler>
{
    fn play_elem_func(
        &self,
    ) -> impl FnMut(PlayElemArgs<'_, Index<Pattern>>) -> Result<(), OverflowError>
    {
        |args| {
            let mut inner_mut = self.inner.borrow_mut();
            let visitor = InterpreterVisitor {
                arenas: self.arenas,
                interval: args.interval,
                offset: args.offset,
                multiplier: args.multiplier,
                inner: RefCell::new(VisitorInner {
                    scheduler: inner_mut.scheduler,
                }),
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

    fn map_cat_or_seq(
        &self,
        multiple: Multiple<Pattern>,
        is_fast: bool,
    ) -> Result<(), OverflowError> {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let make_sim_elem = |elem: Index<Pattern>| {
            Ok(ElemProps {
                elem,
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            })
        };
        let length = CycleTime::checked_from_int(i32::from(multiple.length()))?;
        let get_elements = || {
            multiple
                .iter(self.arenas.get_pattern_chain_arena())
                .map(make_sim_elem)
        };
        play_multiple(
            self.interval,
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

struct VisitorInner<'a, Scheduler> {
    scheduler: &'a mut Scheduler,
}

impl<'a, Arenas: PatternArenas, Scheduler: UnitScheduler> PatternVisitor
    for InterpreterVisitor<'a, Arenas, Scheduler>
{
    type Output = Result<(), OverflowError>;
    type PatternOutput = Result<(), OverflowError>;

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
        self.map_cat_or_seq(multiple, false)
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        self.map_cat_or_seq(multiple, true)
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        // Plays each of the patterns in parallel.

        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        multiple
            .iter(self.arenas.get_pattern_chain_arena())
            .try_for_each(|pattern_index| visit_pattern(self, pattern_index))
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        let multiple_length =
            CycleTime::checked_from_int(i32::from(multiple.length()))?;
        // TODO: Store the total length to reduce repeated calculation
        let total_cycle_length = self
            .timed_step_iter(&multiple)
            .map(|TimedStep(dur, _)| dur)
            .sum::<Result<CycleTime, OverflowError>>()?;
        // If we have a [TimeCat], then we simulate over each one cycle in the
        // [Multiple] and then scale each element individually by its
        // proportion of the total; e.g. TimeCat([1, "A"], [2, "B"], [3, "C"])
        // will have element lengths 1*3/6, 2*3/6, 3*3/6
        // (which adds to 3, the number of elements).
        let get_scaled_length = |elem_length: CycleTime| {
            elem_length
                .mul(multiple_length)?
                .div(total_cycle_length)
        };
        let make_sim_elem = |TimedStep(elem_length, pattern)| {
            Ok(ElemProps {
                elem: pattern,
                sim_duration: CycleTime::ONE,
                played_duration: get_scaled_length(elem_length)?,
            })
        };
        let get_elements = || {
            self.timed_step_iter(&multiple)
                .map(make_sim_elem)
        };
        // Due to fixed point rounding errors, recalculate the total length
        // after calculating the scaled length of each element.
        let played_duration = get_elements()
            .try_fold(CycleTime::ZERO, |acc, x| acc.add(x?.played_duration))?;
        play_multiple(
            self.interval,
            ElemProps {
                elem: get_elements,
                sim_duration: multiple_length,
                played_duration: played_duration,
            },
            true,
            self.offset,
            self.multiplier,
            self.play_elem_func(),
        )
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        if multiple.is_empty() {
            panic!("Cannot play empty multiple patterns");
        }

        // TODO: Store the total length to reduce repeated calculation
        let total_cycle_length = self
            .timed_step_iter(&multiple)
            .map(|TimedStep(dur, _)| dur)
            .sum::<Result<CycleTime, OverflowError>>()?;
        let make_sim_elem = |TimedStep(elem_length, pattern)| {
            Ok(ElemProps {
                elem: pattern,
                sim_duration: elem_length,
                played_duration: elem_length,
            })
        };
        // If we have an [Arrange], then we simulate over all the cycles in
        // the pattern and so the played length is `total_cycle_length`.
        let get_elements = || {
            self.timed_step_iter(&multiple)
                .map(make_sim_elem)
        };
        play_multiple(
            self.interval,
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
        let unit_duration = self.multiplier.recip()?;
        let sound_unit = SoundUnit::new(unit, unit_duration);
        let get_elements = || {
            iter::once(ElemProps {
                elem: sound_unit.clone(),
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            })
            .map(Ok)
        };
        play_multiple(
            self.interval,
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
                if start != start.floor()? {
                    return Ok(());
                }
                let scaled_start = start
                    .add(args.offset)?
                    .div(args.multiplier)?;
                self.inner
                    .borrow_mut()
                    .scheduler
                    .add(args.elem.clone(), scaled_start);
                Ok(())
            },
        )
    }

    fn map_silence(&self) -> Self::PatternOutput {
        Ok(())
    }
}
