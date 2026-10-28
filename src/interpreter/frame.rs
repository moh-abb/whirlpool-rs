use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::leaf::LeafFrame;
use crate::interpreter::query::QueryFrame;
use crate::synth::scheduler::UnitScheduler;

#[derive(derive_more::From, Debug)]
enum FrameInner<'a, Arenas: PatternArenas> {
    Query(QueryFrame),
    Concat(ConcatFrame<'a, Arenas>),
    Leaf(LeafFrame),
}

#[derive(Debug)]
pub struct InterpreterFrame<'a, Arenas: PatternArenas>(FrameInner<'a, Arenas>);

impl<'a, Arenas: PatternArenas, T: Into<FrameInner<'a, Arenas>>> From<T>
    for InterpreterFrame<'a, Arenas>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

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
