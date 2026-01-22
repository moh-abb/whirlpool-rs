use fixed::FixedU32;
use fixed::Wrapping;
use fixed::types::extra::U12;

/// Used to represent cycles, units of time (progression through a
/// [crate::ast::pattern::Pattern]) that are not dependent on the number of
/// beats per minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleTime(pub Wrapping<FixedU32<U12>>);

#[allow(unused)]
impl CycleTime {
    pub const ZERO: Self = Self(Wrapping(FixedU32::ZERO));
    pub const ONE: Self = Self(Wrapping(FixedU32::const_from_int(1)));
}
