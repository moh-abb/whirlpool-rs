use core::str::FromStr;

use chumsky::IterParser;
use chumsky::Parser;
use chumsky::primitive::any;
use chumsky::primitive::just;
use fixed::ParseFixedError;

use crate::ast::CycleTime;
use crate::ast::parser::AstParser;
use crate::ast::parser::Input;
use crate::ast::parser::Parseable;
use crate::ast::parser::array::ParserVec;
use crate::ast::parser::array::ParserVecError;
use crate::ast::time::Inner;

const TIME_STRING_CAPACITY: usize = 16;
type TimeString = ParserVec<u8, TIME_STRING_CAPACITY>;

#[derive(derive_more::From, Debug)]
pub enum ParseCycleTimeError {
    FromAsciiErr(ParseFixedError),
    StringBufferErr(ParserVecError),
}

impl FromStr for CycleTime {
    type Err = ParseCycleTimeError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Inner::from_str(src)
            .map(Self)
            .map_err(ParseCycleTimeError::FromAsciiErr)
    }
}

impl CycleTime {
    fn from_ascii<'src>(
        bytes: &'src [u8],
    ) -> Result<CycleTime, ParseCycleTimeError> {
        Inner::from_ascii(bytes)
            .map(Self)
            .map_err(ParseCycleTimeError::FromAsciiErr)
    }
}

impl<'src> Parseable<'src, ()> for CycleTime {
    type Error = ParseCycleTimeError;

    fn parser<I: Input<'src>>(
        _args: (),
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>> {
        just(b'+')
            .or(just(b'-'))
            .or(just(b'.'))
            .or(any().filter(u8::is_ascii_digit))
            .repeated()
            .at_least(1)
            .collect::<TimeString>()
            .map(|time_str| {
                let slice = time_str.as_slice()?;
                let result = CycleTime::from_ascii(slice)?;
                Ok(result)
            })
    }
}
