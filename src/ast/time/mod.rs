use fixed::FixedI32;
use fixed::types::extra::U12;

use crate::ast::macros::compose_result;

pub mod interval;
pub mod props;
mod traits;

type Inner = FixedI32<U12>;

/// Used to represent cycles, units of time (progression through a
/// [crate::ast::pattern::Pattern]) that are not dependent on the number of
/// beats per minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleTime(Inner);

#[derive(Debug)]
pub struct OverflowError;

macro_rules! checked {
    ($e: expr) => {
        match $e {
            Some(x) => Ok(Self(x)),
            None => Err(OverflowError),
        }
    };
}

macro_rules! unwrapped {
    ($e: expr) => {
        match $e {
            Err(OverflowError) => panic!("Overflow error"),
            Ok(x) => x,
        }
    };
}

impl CycleTime {
    pub const ZERO: Self = Self::unwrapped_from_int(0);
    pub const ONE: Self = Self::unwrapped_from_int(1);

    /// The minimum positive value for which two [CycleTime]s are considered
    /// distinct from one another
    pub const EPSILON: Self = Self(Inner::DELTA);

    #[inline]
    pub const fn unwrapped_from_int(time: i32) -> Self {
        unwrapped!(Self::checked_from_int(time))
    }

    #[inline]
    pub const fn checked_from_int(time: i32) -> Result<Self, OverflowError> {
        const MAX_POS_TIME: i32 = (1_i32 << (Inner::INT_NBITS - 1)) - 1;
        const MAX_NEG_TIME: i32 = -MAX_POS_TIME - 1;
        if time > MAX_POS_TIME || time < MAX_NEG_TIME {
            return Err(OverflowError);
        }
        Ok(Self(Inner::const_from_int(time)))
    }

    #[inline]
    pub const fn to_int(self) -> i32 {
        self.0.int().to_bits() >> Inner::FRAC_NBITS
    }

    #[inline]
    pub const fn from_int_recip(time: i32) -> Self {
        // We assume that if `Self::checked_from_int`, returns None, then `time`
        // is too large (positively or negatively), and so the result is zero.
        assert!(time != 0);
        match Self::checked_from_int(time) {
            Ok(cycle_time) => match cycle_time.recip() {
                Ok(recip) => recip,
                Err(OverflowError) => Self::ZERO,
            },
            Err(OverflowError) => Self::ZERO,
        }
    }

    #[inline]
    #[must_use]
    pub const fn frac(self) -> Self {
        Self(self.0.frac())
    }

    #[inline]
    #[must_use]
    pub const fn neg(self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_neg())
    }

    #[inline]
    #[must_use]
    pub const fn add(self, other: Self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_add(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn sub(self, other: Self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_sub(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn mul(self, other: Self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_mul(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn div(self, other: Self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_div(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn div_euclid(self, other: Self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_div_euclid(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn rem_euclid(self, other: Self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_rem_euclid(other.0))
    }

    #[inline]
    #[must_use]
    pub const fn floor(self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_floor())
    }

    #[inline]
    #[must_use]
    pub const fn ceil(self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_ceil())
    }

    #[inline]
    #[must_use]
    pub const fn recip(self) -> Result<Self, OverflowError> {
        checked!(self.0.checked_recip())
    }

    #[inline]
    #[must_use]
    pub const fn round_up_to_nearest(
        self,
        increment: Self,
    ) -> Result<Self, OverflowError> {
        compose_result!(compose_result!(self.div(increment)).ceil())
            .mul(increment)
    }

    #[inline]
    #[must_use]
    pub const fn round_down_to_nearest(
        self,
        increment: Self,
    ) -> Result<Self, OverflowError> {
        compose_result!(compose_result!(self.div(increment)).floor())
            .mul(increment)
    }

    pub const fn const_le(&self, other: &Self) -> bool {
        self.0.to_bits() <= other.0.to_bits()
    }

    pub const fn const_eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
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
