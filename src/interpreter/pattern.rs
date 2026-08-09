use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::ControlFlow;
use core::ops::DerefMut;

use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::Interpreter;
use crate::interpreter::borrow::BorrowAdapter;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::frame::query_frame;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Index;
use crate::mem::Vec;
use crate::synth::scheduler::UnitScheduler;

/// An interpreter of [Pattern]s, which keeps track of a current time and
/// plays units (traversing the [Pattern]'s tree) when new units are
/// encountered.
pub struct PatternInterpreter<
    'a,
    Arenas,
    Scheduler,
    FrameArena,
    SchedulerBorrow,
    FrameArenaBorrow,
> {
    pattern_index: Index<PatternNode>,
    arenas: &'a Arenas,
    scheduler: SchedulerBorrow,
    frame_arena: FrameArenaBorrow,
    cur_time: CycleTime,
    base_multiplier: CycleTime,
    base_offset: CycleTime,
    phantom: PhantomData<(Scheduler, FrameArena)>,
}

impl<'a, Arenas, Scheduler, FrameArena>
    PatternInterpreter<
        'a,
        Arenas,
        Scheduler,
        FrameArena,
        &'a mut Scheduler,
        &'a mut FrameArena,
    >
{
    /// Constructs an interpreter with the given root index, arenas (to
    /// traverse the pattern structure), scheduler, and starting scope.
    #[allow(unused)]
    pub fn new(
        pattern_index: Index<PatternNode>,
        arenas: &'a Arenas,
        scheduler: &'a mut Scheduler,
        frame_arena: &'a mut FrameArena,
    ) -> Self {
        Self {
            pattern_index,
            arenas,
            scheduler,
            frame_arena,
            cur_time: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas, Scheduler, FrameArena>
    PatternInterpreter<
        'a,
        Arenas,
        Scheduler,
        FrameArena,
        &'a RefCell<Scheduler>,
        &'a RefCell<FrameArena>,
    >
{
    /// Like [Self::new], but with a [RefCell] to allow for a mock scheduler
    /// and mock scope.
    #[allow(unused)]
    pub fn new_with_refcell(
        pattern_index: Index<PatternNode>,
        arenas: &'a Arenas,
        scheduler: &'a RefCell<Scheduler>,
        frame_arena: &'a RefCell<FrameArena>,
    ) -> Self {
        Self {
            pattern_index,
            arenas,
            scheduler,
            frame_arena,
            cur_time: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas, Scheduler, FrameArena, SchedulerBorrow, FrameArenaBorrow>
    PatternInterpreter<
        'a,
        Arenas,
        Scheduler,
        FrameArena,
        SchedulerBorrow,
        FrameArenaBorrow,
    >
{
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

impl<'a, Arenas, Scheduler, FrameArena, SchedulerBorrow, FrameArenaBorrow>
    Interpreter
    for PatternInterpreter<
        'a,
        Arenas,
        Scheduler,
        FrameArena,
        SchedulerBorrow,
        FrameArenaBorrow,
    >
where
    Arenas: PatternArenas,
    Scheduler: UnitScheduler,
    SchedulerBorrow: BorrowAdapter<Scheduler>,
    FrameArenaBorrow: BorrowAdapter<FrameArena>,
{
    type Output = Result<(), PatternInterpreterError>;

    #[allow(unused)]
    #[must_use]
    fn update_time(&mut self, next_time: CycleTime) -> Self::Output {
        // The time should be monotonically increasing.
        debug_assert!(next_time >= self.cur_time);
        let mut borrowed_scheduler = self.scheduler.try_borrow_mut()?;

        // TODO: Replace this with stack-allocated version
        // Perhaps we need to have the stack be part of the interpreter?
        let mut frames = Vec::<InterpreterFrame<'a, Arenas>>::new();
        // Add the start simulation arguments for the pattern.
        frames.push(query_frame(PlayElemArgs {
            elem: self.pattern_index.clone(),
            interval: CycleInterval::new(self.cur_time, next_time),
            offset: self.base_offset,
            multiplier: self.base_multiplier,
        }));

        loop {
            let Some(last_frame) = frames.last_mut() else {
                break;
            };
            let step_result =
                last_frame.step(borrowed_scheduler.deref_mut(), self.arenas)?;
            let opt_new_frame = match step_result {
                ControlFlow::Break(opt_new_frame) => {
                    frames.pop();
                    opt_new_frame
                }
                ControlFlow::Continue(opt_new_frame) => opt_new_frame,
            };
            if let Some(new_frame) = opt_new_frame {
                frames.push(new_frame);
            }
        }

        self.cur_time = next_time;
        Ok(())
    }
}
