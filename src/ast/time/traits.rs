#![deny(unconditional_recursion)]

use core::iter::Sum;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Div;
use core::ops::DivAssign;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Neg;
use core::ops::Rem;
use core::ops::Sub;
use core::ops::SubAssign;

use crate::ast::CycleTime;

macro_rules! impl_unary_op {
    ($tr:ty, $f1: ident, $f2: ident) => {
        impl $tr for CycleTime {
            type Output = Self;
            fn $f1(self) -> Self::Output {
                CycleTime::$f2(self)
            }
        }
    };
}

macro_rules! impl_binary_op {
    ($tr:ty, $f1: ident, $f2: ident) => {
        impl $tr for CycleTime {
            type Output = Self;
            fn $f1(self, rhs: Self) -> Self::Output {
                CycleTime::$f2(self, rhs)
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($tr:ty, $f1: ident, $f2: ident) => {
        impl $tr for CycleTime {
            fn $f1(&mut self, rhs: Self) {
                *self = CycleTime::$f2(*self, rhs)
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
impl_op_assign!(AddAssign, add_assign, add);
impl_op_assign!(SubAssign, sub_assign, sub);
impl_op_assign!(MulAssign, mul_assign, mul);
impl_op_assign!(DivAssign, div_assign, div);

impl Sum<CycleTime> for CycleTime {
    fn sum<I: Iterator<Item = CycleTime>>(iter: I) -> Self {
        iter.fold(CycleTime::ZERO, CycleTime::add)
    }
}
