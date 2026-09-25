use core::str::FromStr;

use combine::Parser;
use combine::choice;
use combine::many1;
use combine::satisfy;
use combine::token;
use fixed::ParseFixedError;

use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::array::ParserVec;
use crate::ast::parser::array::ParserVecError;
use crate::ast::time::CycleTime;
use crate::ast::time::Inner;

const TIME_STRING_CAPACITY: usize = 16;
type TimeString = ParserVec<u8, TIME_STRING_CAPACITY>;

#[derive(derive_more::From, Debug, Clone)]
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

impl CycleTime {
    pub fn parser<I: AstInput>()
    -> impl AstParser<I, Result<Self, ParseCycleTimeError>> + Clone {
        many1::<TimeString, _, _>(choice((
            token(b'+'),
            token(b'-'),
            token(b'.'),
            satisfy(|b: u8| b.is_ascii_digit()),
        )))
        .map(|time_str| {
            let slice = time_str.as_slice()?;
            let result = CycleTime::from_ascii(slice)?;
            Ok(result)
        })
    }
}
