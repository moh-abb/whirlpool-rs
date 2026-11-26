use core::fmt;

use crate::ast::display::AstDisplay;
use crate::ast::display::AstDisplayFlags;

#[derive(Debug, Clone)]
pub enum BuiltinCall {
    Cat,
    Seq,
    Stack,
    TimeCat,
    Arrange,
}

#[derive(derive_more::From, Debug)]
pub enum ResolveBuiltinCallError {
    EmptyInput,
    UnexpectedType,
}

impl BuiltinCall {
    pub fn resolve(bytes: &[u8]) -> Result<Self, ResolveBuiltinCallError> {
        if bytes.is_empty() {
            return Err(ResolveBuiltinCallError::EmptyInput);
        }
        Ok(match bytes {
            b"cat" => Self::Cat,
            b"seq" => Self::Seq,
            b"stack" => Self::Stack,
            b"timecat" | b"timeCat" | b"stepcat" => Self::TimeCat,
            b"arrange" => Self::Arrange,
            _ => return Err(ResolveBuiltinCallError::UnexpectedType),
        })
    }
}

impl fmt::Display for BuiltinCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_ast(f, &AstDisplayFlags::default())
    }
}

impl AstDisplay for BuiltinCall {
    type Error = fmt::Error;

    fn fmt_ast(
        &self,
        f: &mut fmt::Formatter<'_>,
        _flags: &AstDisplayFlags,
    ) -> Result<(), Self::Error> {
        let literal: &str = match self {
            Self::Cat => "cat",
            Self::Seq => "seq",
            Self::Stack => "stack",
            Self::TimeCat => "timecat",
            Self::Arrange => "arrange",
        };

        write!(f, "{}", literal)
    }
}
