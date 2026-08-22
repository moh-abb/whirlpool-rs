use core::str::FromStr;

use chumsky::Parser;
use chumsky::primitive::any;
use chumsky::primitive::just;

use crate::ast::CycleTime;
use crate::ast::parser::Input;
use crate::ast::parser::Parseable;
use crate::ast::time::Inner;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ParseCycleTimeError(<Inner as FromStr>::Err);

impl FromStr for CycleTime {
    type Err = ParseCycleTimeError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Inner::from_str(src)
            .map(Self)
            .map_err(ParseCycleTimeError)
    }
}

impl CycleTime {
    fn from_ascii<'src>(
        bytes: &'src [u8],
    ) -> Result<CycleTime, ParseCycleTimeError> {
        Inner::from_ascii(bytes)
            .map(Self)
            .map_err(ParseCycleTimeError)
    }
}

impl<'src> Parseable<'src, ()> for CycleTime {
    type Error = ParseCycleTimeError;

    fn parser<I: Input<'src>>(
        _args: (),
    ) -> impl Parser<'src, I, Result<Self, Self::Error>> {
        let opt_sign = just(b"+")
            .to(true)
            .or(just(b"-").to(false))
            .or_not();
        let digit = any().filter(|b: &u8| b.is_ascii_digit() || b == &b'.');
        opt_sign
            .then(digit.repeated())
            .to_slice()
            .map(CycleTime::from_ascii)
    }
}
