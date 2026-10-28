use crate::ast::time::CycleTime;

mod borrow;
mod call;
mod concat;
mod elements;
pub mod error;
pub mod frame;
mod iterator;
mod leaf;
pub mod pattern;
pub mod props;
mod query;

pub trait Interpreter {
    type Output;

    fn update_time(&mut self, next_time: CycleTime) -> Self::Output;
}
