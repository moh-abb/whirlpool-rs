use crate::ast::CycleTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleInterval {
    start: CycleTime,
    end: CycleTime,
}

impl CycleInterval {
    pub const fn new(start: CycleTime, end: CycleTime) -> Self {
        debug_assert!(start.const_le(&end));
        Self { start, end }
    }

    pub const fn start(&self) -> CycleTime {
        self.start
    }

    pub const fn end(&self) -> CycleTime {
        self.end
    }

    pub const fn intersection(self, other: Self) -> Option<Self> {
        // There is no intersection between the two intervals if the
        // end of one is before the start of another.
        let start = self.start.const_max(other.start);
        let end = self.end.const_min(other.end);
        if end.const_le(&start) {
            return None;
        }
        Some(Self { start, end })
    }
}
