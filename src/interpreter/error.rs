use crate::ast::time::OverflowError;
use crate::mem::ArenaError;

/// Represents the error type when interpreting a [Pattern].
#[derive(derive_more::From, Clone)]
pub enum PatternInterpreterError {
    ArenaErr(ArenaError),
    OverflowErr(OverflowError),
}
