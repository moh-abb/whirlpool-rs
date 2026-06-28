use crate::ast::CycleTime;

mod borrow;
pub mod elements;
pub mod error;
pub mod pattern;
pub mod props;
pub mod scope;

pub trait Interpreter {
    type Output;

    fn update_time(&mut self, next_time: CycleTime) -> Self::Output;
}
