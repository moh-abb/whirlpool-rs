use crate::ast::pattern::arenas::SumCycleLengthError;
use crate::ast::time::OverflowError;
use crate::mem::ArenaError;
use crate::mem::structures::stack::StackError;

/// Represents the error type when interpreting a [Pattern].
#[derive(derive_more::From, Debug)]
pub enum PatternInterpreterError {
    ArenaErr(ArenaError),
    OverflowErr(OverflowError),
    StackErr(StackError),
    SumCycleLengthErr(SumCycleLengthError),
    MultipleEmpty,
    ExpectedTimedStep,
    ExpectedNormalPattern,
    OverOneChildInTimedStep,
    ExpectedNonemptyTimedStep,
}

pub type PatternInterpreterResult<T> = Result<T, PatternInterpreterError>;
