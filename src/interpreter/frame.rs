use core::ops::ControlFlow;

use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::Index;
use crate::synth::scheduler::UnitScheduler;

pub enum InterpreterFrame<'a, Arenas> {
    Query(QueryFrame),
    Concat(ConcatFrame<'a, Arenas>),
}

/// Indicates the result of interpreting one frame.
/// If we return `Err(err)` then this indicates that interpreting has failed
/// with the given error. Otherwise, `Ok(Continue(None))` indicates to
/// continue stepping without pushing any more frames onto the stack, and
/// `Ok(Continue(Some(frame)))` indicates that `frame` should be pushed onto
/// the stack.
pub type InterpreterResult<'a, Arenas> = PatternInterpreterResult<
    ControlFlow<(), Option<InterpreterFrame<'a, Arenas>>>,
>;

pub trait EvaluateFrame<'a, Arenas> {
    fn step(
        &mut self,
        scheduler: &mut impl UnitScheduler,
        arenas: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas>;
}

pub struct QueryFrame {
    play_args: PlayElemArgs<Index<PatternNode>>,
}

pub fn query_frame<'a, Arenas>(
    play_args: PlayElemArgs<Index<PatternNode>>,
) -> InterpreterFrame<'a, Arenas> {
    InterpreterFrame::Query(QueryFrame { play_args })
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas> for QueryFrame {
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        arenas: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        let index = self.play_args.elem.clone();
        let play_args = self.play_args.map(|_| ());
        let cloned_node = arenas
            .get_pattern_arena()
            .map(index, Clone::clone)?;
        let opt_next_frame = match &cloned_node.pattern {
            Pattern::Cat(multiple) => {
                Some(ConcatFrame::cat_frame(multiple, arenas, play_args)?)
            }
            Pattern::Seq(multiple) => {
                Some(ConcatFrame::seq_frame(multiple, arenas, play_args)?)
            }
            Pattern::Stack(multiple) => {
                Some(ConcatFrame::stack_frame(multiple, arenas, play_args)?)
            }
            Pattern::TimeCat { total_cycle_length, multiple } => todo!(),
            Pattern::Arrange { total_cycle_length, multiple } => todo!(),
            Pattern::TimedStep(timed_step) => todo!(),
            Pattern::Note(note_unit) => todo!(),
            Pattern::Silence => todo!(),
        };
    }
}
