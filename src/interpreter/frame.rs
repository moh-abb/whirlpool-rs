use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::leaf::LeafFrame;
use crate::interpreter::query::QueryFrame;
use crate::interpreter::value::ValueStack;

#[derive(derive_more::From, Debug, Clone)]
enum FrameInner {
    Query(QueryFrame),
    Concat(ConcatFrame),
    Leaf(LeafFrame),
}

#[derive(Debug, Clone)]
pub struct InterpreterFrame(FrameInner);

impl<T: Into<FrameInner>> From<T> for InterpreterFrame {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

/// Indicates the result of interpreting one frame.
/// If we return `Err(err)`, then interpretation has failed with the given
/// error. Otherwise `Ok(None)` indicates that interpretation of the current
/// frame is complete, and `Ok(Some(frame))` indicates to keep this frame on
/// the stack and interpret the new one on top.
pub type InterpreterResult = PatternInterpreterResult<Option<InterpreterFrame>>;

pub(super) trait EvaluateFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult;
}

impl EvaluateFrame for InterpreterFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        // TODO: Automatically delegate this step.
        match &mut self.0 {
            FrameInner::Query(frame) => frame.step(value_stack, arenas),
            FrameInner::Concat(frame) => frame.step(value_stack, arenas),
            FrameInner::Leaf(frame) => frame.step(value_stack, arenas),
        }
    }
}
