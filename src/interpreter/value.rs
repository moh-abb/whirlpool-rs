use crate::ast::pattern::TimedStep;
use crate::ast::time::CycleTime;
use crate::interpreter::props::ElemProps;
use crate::mem::structures::stack::StackError;
use crate::synth::unit::SoundUnit;

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
