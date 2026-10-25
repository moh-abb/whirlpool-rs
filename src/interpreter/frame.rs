use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::leaf::LeafFrame;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Index;
use crate::synth::scheduler::UnitScheduler;

#[derive(derive_more::From, Debug)]
enum FrameInner<'a, Arenas: PatternArenas> {
    Query(QueryFrame),
    Concat(ConcatFrame<'a, Arenas>),
    Leaf(LeafFrame),
}

#[derive(Debug)]
pub struct InterpreterFrame<'a, Arenas: PatternArenas>(FrameInner<'a, Arenas>);

/// Indicates the result of interpreting one frame.
/// If we return `Err(err)`, then interpretation has failed with the given
/// error. Otherwise `Ok(None)` indicates that interpretation of the current
/// frame is complete, and `Ok(Some(frame))` indicates to keep this frame on
/// the stack and interpret the new one on top.
pub type InterpreterResult<'a, Arenas> =
    PatternInterpreterResult<Option<InterpreterFrame<'a, Arenas>>>;

pub(super) trait EvaluateFrame<'a, Arenas: PatternArenas> {
    fn step(
        &mut self,
        scheduler: &mut impl UnitScheduler,
        arenas: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas>;
}

#[derive(Debug)]
pub(super) struct QueryFrame {
    play_args: PlayElemArgs<Index<PatternNode>>,
    finished: bool,
}

pub(super) fn query_frame<'a, Arenas: PatternArenas>(
    play_args: PlayElemArgs<Index<PatternNode>>,
) -> InterpreterFrame<'a, Arenas> {
    InterpreterFrame((QueryFrame { play_args, finished: false }).into())
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas> for QueryFrame {
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        arenas: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        // Check if we are done.
        if self.finished {
            return Ok(None);
        }
        let index = self.play_args.elem.clone();
        let play_args = self.play_args.clone().map(|_| ());
        let cloned_pattern = arenas.map(index, Clone::clone)?.pattern;
        let next_frame: FrameInner<'_, _> = match &cloned_pattern {
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
            Pattern::Arrange { total_cycle_length, multiple } => {
                ConcatFrame::arrange_frame(
                    multiple,
                    total_cycle_length,
                    arenas,
                    play_args,
                )?
                .into()
            }
            Pattern::TimedStep(_) => {
                // We shouldn't be querying a timed step without using the
                // parent TimeCat or Arrange.
                return Err(PatternInterpreterError::ExpectedNormalPattern);
            }
            Pattern::Note(note_unit) => {
                LeafFrame::note_frame(note_unit, play_args)?.into()
            }
            Pattern::Silence => LeafFrame::silence()?.into(),
        };
        // Indicate that we will delete this frame after the child frame exits.
        self.finished = true;
        Ok(Some(InterpreterFrame(next_frame)))
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for InterpreterFrame<'a, Arenas>
{
    fn step(
        &mut self,
        scheduler: &mut impl UnitScheduler,
        arenas: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        // TODO: Automatically delegate this step.
        match &mut self.0 {
            FrameInner::Query(frame) => frame.step(scheduler, arenas),
            FrameInner::Concat(ConcatFrame::CatOrSeq(frame)) => {
                frame.step(scheduler, arenas)
            }
            FrameInner::Concat(ConcatFrame::Stack(frame)) => {
                frame.step(scheduler, arenas)
            }
            FrameInner::Concat(ConcatFrame::TimeCat(frame)) => {
                frame.step(scheduler, arenas)
            }
            FrameInner::Concat(ConcatFrame::Arrange(frame)) => {
                frame.step(scheduler, arenas)
            }
            FrameInner::Leaf(LeafFrame::Note(frame)) => {
                frame.step(scheduler, arenas)
            }
            FrameInner::Leaf(LeafFrame::Silence(frame)) => {
                frame.step(scheduler, arenas)
            }
        }
    }
}
