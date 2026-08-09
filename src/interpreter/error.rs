use crate::ast::time::OverflowError;
use crate::mem::ArenaError;

/// Represents the error type when interpreting a [Pattern].
#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternInterpreterError {
    ArenaErr(ArenaError),
    Overflow(OverflowError),
    MultipleEmpty,
    ExpectedTimedStep,
    ExpectedNormalPattern,
}

pub type PatternInterpreterResult<T> = Result<T, PatternInterpreterError>;
