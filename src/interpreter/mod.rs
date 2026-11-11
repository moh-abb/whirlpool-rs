use crate::ast::time::CycleTime;

mod borrow;
mod call;
mod concat;
pub mod error;
pub mod frame;
mod leaf;
pub mod pattern;
pub mod props;
mod query;
mod repeat;
pub mod value;

pub trait Interpreter {
    type Output;

    fn update_time(&mut self, next_time: CycleTime) -> Self::Output;
}
