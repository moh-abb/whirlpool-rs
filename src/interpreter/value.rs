use crate::ast::expr::ExprNode;
use crate::ast::time::CycleTime;
use crate::interpreter::props::ElemProps;
use crate::mem::Index;
use crate::mem::structures::stack::StackError;
use crate::synth::unit::SoundUnit;

/// Represents a [Pattern] played with a given duration, and a child expression.
/// Due to our relaxed AST, we only really use this while interpreting the
/// patterns.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimedStep(pub CycleTime, pub Index<ExprNode>);

/// Represents different objects that can be placed onto the value stack.
#[derive(derive_more::TryInto, derive_more::From, Debug)]
pub enum Value {
    SoundUnit(SoundUnit),
    TimedStep(TimedStep),
    CycleTime(CycleTime),
    TotalProps(ElemProps<()>),
}

#[cfg_attr(test, mockall::automock)]
pub trait ValueStack {
    fn push<T>(&mut self, item: T) -> Result<(), ValueStackError>
    where
        T: Into<Value> + 'static;

    fn pop<T>(&mut self) -> Result<T, ValueStackError>
    where
        T: 'static,
        Value: TryInto<T>;
}

#[derive(Debug)]
pub enum ValueStackError {
    StackErr(StackError),
    StackEmpty,
    TypeMismatch,
}
