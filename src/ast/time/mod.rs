use fixed::FixedI32;
use fixed::types::extra::U12;

pub mod interval;
pub mod props;
mod traits;

type Inner = FixedI32<U12>;

/// Used to represent cycles, units of time (progression through a
/// [crate::ast::pattern::Pattern]) that are not dependent on the number of
/// beats per minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleTime(Inner);

impl CycleTime {
    pub const ZERO: Self = Self(FixedI32::ZERO);
    pub const ONE: Self = Self::from_int(1);

    #[inline]
    pub const fn from_int(time: i32) -> Self {
        Self(FixedI32::const_from_int(time))
    }

    #[inline]
    pub const fn checked_from_int(time: i32) -> Option<Self> {
        const MAX_POS_TIME: i32 = (1_i32 << (Inner::INT_NBITS - 1)) - 1;
        const MAX_NEG_TIME: i32 = -MAX_POS_TIME - 1;
        if time > MAX_POS_TIME || time < MAX_NEG_TIME {
            return None;
        }
        Some(Self::from_int(time))
    }

    #[inline]
    pub const fn to_int(self) -> i32 {
        self.0.int().to_bits() >> Inner::FRAC_NBITS
    }

    #[inline]
    pub const fn from_int_recip(time: i32) -> Self {
        // We assume that if `Self::checked_from_int`, returns None, then `time`
        // is too large (positively or negatively), and so the result is zero.
        match Self::checked_from_int(time) {
            Some(cycle_time) => cycle_time.recip(),
            None => CycleTime::ZERO,
        }
    }

    #[inline]
    #[must_use]
    pub const fn frac(self) -> Self {
        Self(self.0.frac())
    }

    #[inline]
    #[must_use]
    pub const fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }

    #[inline]
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn mul(self, other: Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn div(self, other: Self) -> Self {
        Self(self.0.saturating_div(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn div_euclid(self, other: Self) -> Self {
        Self(self.0.saturating_div_euclid(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn rem_euclid(self, other: Self) -> Self {
        Self(self.0.rem_euclid(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn floor(self) -> Self {
        Self(self.0.saturating_floor())
    }

    #[inline]
    #[must_use]
    pub const fn ceil(self) -> Self {
        Self(self.0.saturating_ceil())
    }

    #[inline]
    #[must_use]
    pub const fn recip(self) -> Self {
        Self(self.0.saturating_recip())
    }

    #[inline]
    #[must_use]
    pub const fn round_up_to_nearest(self, increment: Self) -> Self {
        self.div(increment)
            .ceil()
            .mul(increment)
    }

    #[inline]
    #[must_use]
    pub const fn round_down_to_nearest(self, increment: Self) -> Self {
        self.div(increment)
            .floor()
            .mul(increment)
    }

    pub const fn const_le(&self, other: &Self) -> bool {
        self.0.to_bits() <= other.0.to_bits()
    }

    #[inline]
    #[must_use]
    pub const fn const_max(self, other: Self) -> Self {
        if self.const_le(&other) { other } else { self }
    }

    #[inline]
    #[must_use]
    pub const fn const_min(self, other: Self) -> Self {
        if self.const_le(&other) { self } else { other }
    }
}
