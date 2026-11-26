use crate::ast::expr::builtin::ResolveBuiltinCallError;
use crate::ast::note::resolve::ResolveNoteError;
use crate::ast::parser::array::ParserVecError;
use crate::ast::time::OverflowError;
use crate::interpreter::value::ValueStackError;
use crate::mem::ArenaError;
use crate::mem::structures::stack::StackError;

/// Represents the error type when interpreting a [Pattern].
#[derive(derive_more::From, Debug)]
pub enum PatternInterpreterError {
    ArenaErr(ArenaError),
    OverflowErr(OverflowError),
    StackErr(StackError),
    ParserVecErr(ParserVecError),
    ResolveNoteErr(ResolveNoteError),
    ResolveBuiltinCallErr(ResolveBuiltinCallError),
    ValueStackErr(ValueStackError),
    MultipleEmpty,
    ExpectedTimedStep,
    ExpectedNormalPattern,
    ExpectedNonemptyTimedStep,
    ExpectedTwoElementsInTimedStep,
    UnexpectedPopElemProps,
    UnexpectedIdentInQuery,
    UnexpectedArrayInQuery,
    UnexpectedLambdaInQuery,
    UnexpectedNumberInQuery,
    UnexpectedStringLiteralInQuery,
    InvalidTimedStepTime,
}

pub type PatternInterpreterResult<T> = Result<T, PatternInterpreterError>;
