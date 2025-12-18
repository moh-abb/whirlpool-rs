#[cfg(test)]
use proptest_derive::Arbitrary;

use super::note::NoteUnit;
use crate::arena::chain::Chain;
use crate::arena::index::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TimeUnit(pub u32);

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimedStep(pub TimeUnit, pub Index<Pattern>);

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pattern {
    Cat(Chain<Self>),
    Seq(Chain<Self>),
    Stack(Chain<Self>),
    TimeCat(Chain<TimedStep>),
    Note(NoteUnit),
    Silence,
}
