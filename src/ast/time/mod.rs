use fixed::FixedU32;
use fixed::Wrapping;
use fixed::types::extra::U12;

/// Used to represent cycles, units of time (progression through a
/// [crate::ast::pattern::Pattern]) that are not dependent on the number of
/// beats per minute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(unused)]
pub struct CycleTime(pub Wrapping<FixedU32<U12>>);
