use fixed::FixedI32;
use fixed::Wrapping;
use fixed::types::extra::U12;

/// Used to represent cycles, units of time (progression through a
/// [crate::ast::pattern::Pattern]) that are not dependent on the number of
/// beats per minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleTime(pub Wrapping<FixedI32<U12>>);

impl CycleTime {
    pub const ZERO: Self = Self(Wrapping(FixedI32::ZERO));
    pub const ONE: Self = Self::from_int(1);

    pub const fn from_int(time: i32) -> Self {
        Self(Wrapping(FixedI32::const_from_int(time)))
    }

    pub const fn from_int_recip(time: i32) -> Self {
        Self(Wrapping(Self::from_int(time).0.0.recip()))
    }

    pub const fn const_add(self, other: Self) -> Self {
        Self(Wrapping(self.0.0.wrapping_add(other.0.0)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleTimeInterval {
    start: CycleTime,
    end: CycleTime,
}

impl CycleTimeInterval {
    pub fn new(start: CycleTime, end: CycleTime) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    pub const fn start(&self) -> CycleTime {
        self.start
    }

    pub const fn end(&self) -> CycleTime {
        self.end
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        // There is no intersection between the two intervals if the
        // end of one is before the start of another.
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start >= end {
            return None;
        }
        Some(Self { start, end })
    }
}
