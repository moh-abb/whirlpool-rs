//! Defines parser input and actual parser traits.

pub use chumsky::Parser as ChumskyParser;
use chumsky::input::Input;
use chumsky::input::SliceInput;
use chumsky::input::ValueInput;
use chumsky::span::SimpleSpan;

pub trait ParserInput<'src>
where
    Self: Input<'src, Token = u8, Span = SimpleSpan>
        + ValueInput<'src>
        + SliceInput<'src, Slice = &'src [u8]>,
{
}

impl<'src, I> ParserInput<'src> for I where
    Self: Input<'src, Token = u8, Span = SimpleSpan>
        + ValueInput<'src>
        + SliceInput<'src, Slice = &'src [u8]>
{
}

pub trait Parser<'src, I, O>
where
    I: ParserInput<'src>,
    Self: ChumskyParser<'src, I, O>,
{
}

impl<'src, I, O, P> Parser<'src, I, O> for P
where
    I: ParserInput<'src>,
    Self: ChumskyParser<'src, I, O>,
{
}

pub trait Parseable<'src, Args>: Sized {
    type Error;

    fn parser<I: ParserInput<'src>>(
        args: Args,
    ) -> impl Parser<'src, I, Result<Self, Self::Error>>;
}
