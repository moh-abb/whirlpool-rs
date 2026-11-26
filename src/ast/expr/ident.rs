use core::fmt::Write;
use core::fmt::{self};

use combine::Parser;
use combine::many;
use combine::one_of;
use combine::parser::byte::alpha_num;
use combine::parser::byte::letter;

use crate::ast::display::AstDisplay;
use crate::ast::display::AstDisplayFlags;
use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::array::ParserVec;
use crate::ast::parser::array::ParserVecError;
use crate::mem::ArenaError;
use crate::mem::debug_unwrap;

#[derive(
    derive_more::AsRef,
    derive_new::new,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Identifier {
    bytes: ParserVec<u8, { Self::MAX_LENGTH }>,
}

impl FromIterator<u8> for Identifier {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut bytes = ParserVec::new();
        bytes.extend(iter);
        Self { bytes }
    }
}

#[derive(derive_more::From, Debug, Clone)]
pub enum ParseIdentError {
    ParserVecErr(ParserVecError),
}

impl Identifier {
    pub const MAX_LENGTH: usize = 20;

    pub fn parser<I: AstInput>()
    -> impl AstParser<I, Result<Self, ParseIdentError>> {
        let first = letter().or(one_of(b"_~".iter().cloned()));
        let rest = alpha_num().or(one_of(b"_~#^".iter().cloned()));
        (first, many::<ParserVec<u8, { Self::MAX_LENGTH }>, _, _>(rest))
            .map(|(x, xs)| {
                let mut ys = ParserVec::new();
                ys.extend([x]);
                ys.extend(xs.as_slice()?.iter().cloned());
                Ok(Self { bytes: ys })
            })
            .expected("identifier")
    }
}

#[derive(derive_more::From, Debug)]
pub enum DisplayIdentError {
    ArenaErr(ArenaError),
    FormatErr(fmt::Error),
    ParserVecErr(ParserVecError),
    InvalidByte(u8),
}

impl AstDisplay for Identifier {
    type Error = DisplayIdentError;

    fn fmt_ast(
        &self,
        f: &mut fmt::Formatter<'_>,
        _flags: &AstDisplayFlags,
    ) -> Result<(), Self::Error> {
        self.as_ref()
            .as_slice()?
            .iter()
            .try_for_each(|&b| {
                let c = char::from_u32(u32::from(b))
                    .ok_or(DisplayIdentError::InvalidByte(b))?;
                f.write_char(c)?;
                Ok(())
            })
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_unwrap(self.fmt_ast(f, &AstDisplayFlags::default()));
        Ok(())
    }
}
