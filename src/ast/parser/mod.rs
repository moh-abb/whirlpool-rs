//! Defines parser input and actual parser traits.

use chumsky::Parser as ChumskyParser;
use chumsky::input::Input as ChumskyInput;
use chumsky::input::SliceInput;
use chumsky::input::ValueInput;
use chumsky::span::SimpleSpan;

pub mod array;

pub trait Input<'src>
where
    Self: ChumskyInput<'src, Token = u8, Span = SimpleSpan>
        + ValueInput<'src>
        + SliceInput<'src, Slice = &'src [u8]>,
{
}

impl<'src, I> Input<'src> for I where
    Self: ChumskyInput<'src, Token = u8, Span = SimpleSpan>
        + ValueInput<'src>
        + SliceInput<'src, Slice = &'src [u8]>
{
}

pub trait AstParser<'src, I, O>
where
    I: Input<'src>,
    Self: ChumskyParser<'src, I, O>,
{
}

impl<'src, I, O, P> AstParser<'src, I, O> for P
where
    I: Input<'src>,
    Self: ChumskyParser<'src, I, O>,
{
}

pub trait Parseable<'src, Args>: Sized {
    type Error;

    fn parser<I: Input<'src>>(
        args: Args,
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>>;
}
