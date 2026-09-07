pub mod display;
pub mod note;
pub mod parser;
pub mod pattern;
pub mod time;

mod macros {
    macro_rules! compose_result {
        ($x: expr) => {
            match $x {
                Err(e) => return Err(e),
                Ok(x_inner) => x_inner,
            }
        };
    }

    pub(crate) use compose_result;
}
