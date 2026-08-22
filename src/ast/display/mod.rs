use core::fmt;

#[derive(Debug, Default, Clone)]
pub struct AstDisplayFlags {}

pub trait AstDisplay: fmt::Display {
    type Error: From<fmt::Error>;

    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
        _flags: &AstDisplayFlags,
    ) -> Result<(), Self::Error> {
        <Self as fmt::Display>::fmt(self, f)?;
        Ok(())
    }
}
