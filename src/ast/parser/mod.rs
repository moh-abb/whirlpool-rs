//! Defines parser input and actual parser traits.

pub use chumsky::Parser as ChumskyParser;
use chumsky::input::ValueInput;
use chumsky::span::SimpleSpan;

pub trait ParserInput<'src>
where
    Self: ValueInput<'src, Token = char, Span = SimpleSpan>,
{
}

impl<'src, I> ParserInput<'src> for I where
    Self: ValueInput<'src, Token = char, Span = SimpleSpan>
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
