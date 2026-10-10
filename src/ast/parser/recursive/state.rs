use combine::stream::ResetStream;
use derive_where::derive_where;

use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::recursive::error::RecResult;
use crate::mem::Index;

/// Stores the data needed for one level of a recursive parse.
pub struct RecFrame<I, T, P>
where
    I: AstInput,
    P: AstParser<I, RecResult<T>>,
{
    pub(super) parser: P,
    pub(super) state: P::PartialState,
    pub(super) output_index: Index<RecOutput<I, T>>,
}

/// Stores the output of one level of a recursive parse.
#[derive(derive_more::Debug)]
#[debug(bound())]
#[derive_where(PartialEq, Eq, PartialOrd, Ord)]
pub enum RecOutput<I: AstInput, T> {
    Start,

    Pending(
        #[debug(skip)]
        #[derive_where(skip)]
        <I as ResetStream>::Checkpoint,
    ),

    Recursing,

    Complete(
        #[debug(skip)]
        #[derive_where(skip)]
        T,
    ),
}
