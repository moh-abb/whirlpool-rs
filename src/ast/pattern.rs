use crate::arena::chain::Chain;
use crate::arena::index::Index;
use super::note::NoteUnit;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TimeUnit(pub u32);

#[allow(dead_code)]
#[derive(Debug)]
pub struct TimedStep(pub TimeUnit, pub Index<Pattern>);

#[allow(dead_code)]
#[derive(Debug)]
pub enum Pattern {
    Cat(Chain<Self>),
    Seq(Chain<Self>),
    Stack(Chain<Self>),
    TimeCat(Chain<TimedStep>),
    Note(NoteUnit),
    Silence,
}
