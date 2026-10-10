use crate::ast::parser::recursive::RecIterations;
use crate::mem::ArenaError;
use crate::mem::structures::stack::StackError;

#[derive(derive_more::From, Debug, Clone)]
pub enum RecError {
    CallbackFailed,
    ResetFailed,
    StackEmpty,
    MaximumIterations(RecIterations),
    StackErr(StackError),
    ArenaErr(ArenaError),
}

pub type RecResult<T> = Result<T, RecError>;
