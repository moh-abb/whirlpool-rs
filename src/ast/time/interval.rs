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

    pub const fn const_eq(&self, other: &Self) -> bool {
        self.start.const_eq(&other.start) && self.end.const_eq(&other.end)
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

    /// Calculates an interval with each end as a linear interpolation inside
    /// `self`, interpolated by the same proportion that the corresponding end
    /// of `inner` is along `outer`.
    ///
    /// For example, if inner = [2, 4), outer = [0, 8), self = [2, 6),
    /// then the result is [3, 4).
    #[inline]
    pub const fn lerp_interval(
        self,
        inner: CycleInterval,
        outer: CycleInterval,
    ) -> Option<CycleInterval> {
        // Let  outer_length = outer.end - outer.start
        //      alpha     = (inner.start - outer.start) / outer_length
        //      1 - alpha = (outer.end - inner.start)   / outer_length
        //      beta      = (inner.end - outer.start)   / outer_length
        //      1 - beta  = (outer.end - inner.end)     / outer_length
        // Then with lerp(p, x, y) = p * y + (1 - p) * x,
        // y.start = lerp(alpha, self.start, self.end)
        //         = (
        //             inner.start * (self.end - self.start)
        //             + (self.start * outer.end - self.end * outer.start)
        //           ) / outer_length
        //         = (inner.start * self_length + k) / outer_length
        // y.end = lerp(beta, self.start, self.end)
        //       = (
        //           inner.end * (self.end - self.start)
        //           + (self.start * outer.end - self.end * outer.start)
        //         ) / outer_length
        //       = (inner.end * self_length + k) / outer_length
        // where k = self.start * outer.end - self.end * outer.start
        //
        // Trivially, if inner = outer then the result is self
        // (in which case alpha = 0, beta = 1).
        if inner.const_eq(&outer) {
            return Some(self);
        }
        let outer_length = outer.end().sub(outer.start());
        let self_length = self.end().sub(self.start());
        let k = self
            .start()
            .mul(outer.end())
            .sub(self.end().mul(outer.start()));

        const fn endpoint(
            point: CycleTime,
            input_length: CycleTime,
            outer_length: CycleTime,
            k: CycleTime,
        ) -> CycleTime {
            point
                .mul(input_length)
                .add(k)
                .div(outer_length)
        }

        let start = endpoint(inner.start(), self_length, outer_length, k)
            .const_max(self.start());
        let end = endpoint(inner.end(), self_length, outer_length, k)
            .const_min(self.end());
        if end.const_le(&start) {
            return None;
        }
        Some(CycleInterval::new(start, end))
    }
}
