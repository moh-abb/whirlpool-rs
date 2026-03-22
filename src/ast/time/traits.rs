#![deny(unconditional_recursion)]

use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Neg;
use core::ops::Rem;
use core::ops::Sub;

use crate::ast::time::CycleTime;

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

impl_binary_op!(Add, add, add);
impl_binary_op!(Sub, sub, sub);
impl_binary_op!(Mul, mul, mul);
impl_binary_op!(Div, div, div);
impl_binary_op!(Rem, rem, rem_euclid);
impl_unary_op!(Neg, neg, neg);
