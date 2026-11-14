use core::fmt::Write;
use core::fmt::{self};

use crate::ast::display::AstDisplay;
use crate::ast::display::AstDisplayFlags;
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

impl Identifier {
    pub const MAX_LENGTH: usize = 20;
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
