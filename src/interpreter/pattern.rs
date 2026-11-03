use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::CycleTime;
use crate::ast::time::interval::CycleInterval;
use crate::interpreter::Interpreter;
use crate::interpreter::borrow::BorrowAdapter;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::query::QueryFrame;
use crate::interpreter::value::ValueStack;
use crate::mem::Arena;
use crate::mem::Index;
use crate::mem::Stack;
use crate::mem::structures::stack::StackChain;

/// An interpreter of [Pattern]s, which keeps track of a current time and
/// plays units (traversing the [Pattern]'s tree) when new units are
/// encountered.
pub struct PatternInterpreter<
    'a,
    Arenas,
    ValStack,
    FrameArena,
    ValStackBorrow,
    FrameArenaBorrow,
> {
    pattern_index: Index<PatternNode>,
    arenas: &'a Arenas,
    value_stack: ValStackBorrow,
    frame_arena: FrameArenaBorrow,
    cur_time: CycleTime,
    base_multiplier: CycleTime,
    base_offset: CycleTime,
    phantom: PhantomData<(ValStack, FrameArena)>,
}

impl<'a, Arenas, ValStack, FrameArena>
    PatternInterpreter<
        'a,
        Arenas,
        ValStack,
        FrameArena,
        &'a mut ValStack,
        &'a mut FrameArena,
    >
{
    /// Constructs an interpreter with the given root index, arenas (to
    /// traverse the pattern structure), value stack, and starting scope.
    #[allow(unused)]
    pub fn new(
        pattern_index: Index<PatternNode>,
        arenas: &'a Arenas,
        value_stack: &'a mut ValStack,
        frame_arena: &'a mut FrameArena,
    ) -> Self {
        Self {
            pattern_index,
            arenas,
            value_stack,
            frame_arena,
            cur_time: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas, ValStack, FrameArena>
    PatternInterpreter<
        'a,
        Arenas,
        ValStack,
        FrameArena,
        &'a RefCell<ValStack>,
        &'a RefCell<FrameArena>,
    >
{
    /// Like [Self::new], but with a [RefCell] to allow for a mock value stack
    /// and mock scope.
    #[allow(unused)]
    pub fn new_with_refcell(
        pattern_index: Index<PatternNode>,
        arenas: &'a Arenas,
        value_stack: &'a RefCell<ValStack>,
        frame_arena: &'a RefCell<FrameArena>,
    ) -> Self {
        Self {
            pattern_index,
            arenas,
            value_stack,
            frame_arena,
            cur_time: CycleTime::ZERO,
            base_multiplier: CycleTime::ONE,
            base_offset: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas, ValStack, FrameArena, ValStackBorrow, FrameArenaBorrow>
    PatternInterpreter<
        'a,
        Arenas,
        ValStack,
        FrameArena,
        ValStackBorrow,
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

impl<'a, Arenas, ValStack, FrameArena, ValStackBorrow, FrameArenaBorrow>
    Interpreter
    for PatternInterpreter<
        'a,
        Arenas,
        ValStack,
        FrameArena,
        ValStackBorrow,
        FrameArenaBorrow,
    >
where
    Arenas: PatternArenas,
    ValStack: ValueStack,
    ValStackBorrow: BorrowAdapter<ValStack>,
    FrameArenaBorrow: BorrowAdapter<FrameArena>,
    FrameArena: Arena<StackChain<InterpreterFrame<'a, Arenas>>>,
{
    type Output = Result<(), PatternInterpreterError>;

    #[allow(unused)]
    #[must_use]
    fn update_time(&mut self, next_time: CycleTime) -> Self::Output {
        // The time should be monotonically increasing.
        debug_assert!(next_time >= self.cur_time);
        let mut borrowed_value_stack = self.value_stack.try_borrow_mut()?;
        let mut borrowed_arena = self.frame_arena.try_borrow_mut()?;
        let mut frames = Stack::new(borrowed_arena.deref_mut());
        // Add the start simulation arguments for the pattern.
        frames.push(
            QueryFrame::new(PlayElemArgs {
                elem: self.pattern_index.clone(),
                interval: CycleInterval::new(self.cur_time, next_time),
                offset: self.base_offset,
                multiplier: self.base_multiplier,
            })
            .into(),
        )?;

        loop {
            let opt_step_result = frames.map_mut(|frame| {
                frame.step(borrowed_value_stack.deref_mut(), self.arenas)
            })?;
            let Some(step_result) = opt_step_result else {
                // The stack is empty, so we are done.
                break;
            };

            if let Some(new_frame) = step_result? {
                frames.push(new_frame)?;
            } else {
                frames.pop()?;
            }
        }

        self.cur_time = next_time;
        Ok(())
    }
}
