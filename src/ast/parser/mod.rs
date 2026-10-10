//! Defines parser input and actual parser traits.

use combine::Parser;
use combine::Stream;

pub(super) mod array;
pub(super) mod collect;
pub mod recursive;

pub trait AstInput
where
    Self: Stream<Token = u8> + Clone,
{
}

impl<I> AstInput for I where Self: Stream<Token = u8> + Clone {}

pub trait AstParser<I, O>
where
    I: AstInput,
    Self: Parser<I, Output = O>,
{
}

impl<'src, I, O, P> AstParser<I, O> for P
where
    I: AstInput,
    Self: Parser<I, Output = O>,
{
}
