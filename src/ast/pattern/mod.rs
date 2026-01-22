use core::cmp::Ordering;

#[cfg(test)]
use proptest_derive::Arbitrary;

use crate::arena::index::Index;
use crate::ast::multiple::Multiple;

pub mod arenas;
pub mod clone;
pub mod drop;
pub mod equality;
pub mod format_display;
pub mod interpreter;
pub mod note;
#[cfg(test)]
pub mod string_display;
mod visitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TimeUnit(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimedStep(pub TimeUnit, pub Index<Pattern>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Cat(Multiple<Self>),
    Seq(Multiple<Self>),
    Stack(Multiple<Self>),
    TimeCat(Multiple<TimedStep>),
    Note(note::NoteUnit),
    Silence,
}

const fn pattern_discriminant(pattern: &Pattern) -> u8 {
    match pattern {
        Pattern::Cat(_) => 1,
        Pattern::Seq(_) => 2,
        Pattern::Stack(_) => 3,
        Pattern::TimeCat(_) => 4,
        Pattern::Note(_) => 5,
        Pattern::Silence => 6,
    }
}

impl PartialOrd for Pattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pattern {
    fn cmp(&self, other: &Self) -> Ordering {
        let discriminant_order =
            pattern_discriminant(self).cmp(&pattern_discriminant(other));
        if discriminant_order != Ordering::Equal {
            return discriminant_order;
        }
        match (self, other) {
            (Self::Cat(lhs), Self::Cat(rhs)) => lhs.cmp(rhs),
            (Self::Seq(lhs), Self::Seq(rhs)) => lhs.cmp(rhs),
            (Self::Stack(lhs), Self::Stack(rhs)) => lhs.cmp(rhs),
            (Self::TimeCat(lhs), Self::TimeCat(rhs)) => lhs.cmp(rhs),
            (Self::Note(lhs), Self::Note(rhs)) => lhs.cmp(rhs),
            (Self::Silence, Self::Silence) => Ordering::Equal,
            _ => unreachable!(),
        }
    }
}
