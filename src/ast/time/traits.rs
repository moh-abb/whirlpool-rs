#![deny(unconditional_recursion)]

use core::fmt;
use core::iter::Sum;
use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Neg;
use core::ops::Rem;
use core::ops::Sub;

use crate::ast::CycleTime;
use crate::ast::time::OverflowError;

macro_rules! impl_unary_op {
    ($tr:ty, $f1: ident, $f2: ident) => {
        impl $tr for CycleTime {
            type Output = Self;
            fn $f1(self) -> Self::Output {
                CycleTime::$f2(self).unwrap()
            }
        }
    };
}

macro_rules! impl_binary_op {
    ($tr:ty, $f1: ident, $f2: ident) => {
        impl $tr for CycleTime {
            type Output = Self;
            fn $f1(self, rhs: Self) -> Self::Output {
                CycleTime::$f2(self, rhs).unwrap()
            }
        }
    };
}

impl_binary_op!(Add, add, add);
impl_binary_op!(Sub, sub, sub);
impl_binary_op!(Mul, mul, mul);
impl_binary_op!(Div, div, div);
impl_binary_op!(Rem, rem, rem_euclid);
impl_unary_op!(Neg, neg, neg);

impl Sum<CycleTime> for Result<CycleTime, OverflowError> {
    fn sum<I: Iterator<Item = CycleTime>>(mut iter: I) -> Self {
        iter.try_fold(CycleTime::ZERO, CycleTime::add)
    }
}

impl fmt::Display for CycleTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
