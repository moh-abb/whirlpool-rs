use core::ops::ControlFlow;

use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::leaf::LeafFrame;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::Index;
use crate::synth::scheduler::UnitScheduler;

#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterpreterFrame<'a, Arenas> {
    Query(QueryFrame),
    Concat(ConcatFrame<'a, Arenas>),
    Leaf(LeafFrame),
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
        let next_frame: InterpreterFrame<'_, _> = match &cloned_node.pattern {
            Pattern::Cat(multiple) => {
                ConcatFrame::cat_frame(multiple, arenas, play_args)?.into()
            }
            Pattern::Seq(multiple) => {
                ConcatFrame::seq_frame(multiple, arenas, play_args)?.into()
            }
            Pattern::Stack(multiple) => {
                ConcatFrame::stack_frame(multiple, arenas, play_args)?.into()
            }
            Pattern::TimeCat { total_cycle_length, multiple } => {
                ConcatFrame::time_cat_frame(
                    multiple,
                    total_cycle_length,
                    arenas,
                    play_args,
                )?
                .into()
            }
            Pattern::Arrange { total_cycle_length, multiple } => todo!(),
            Pattern::TimedStep(timed_step) => todo!(),
            Pattern::Note(note_unit) => {
                LeafFrame::note_frame(note_unit, play_args)?.into()
            }
            Pattern::Silence => LeafFrame::silence().into(),
        };
        todo!()
    }
}
