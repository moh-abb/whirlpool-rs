#![allow(unused)]
//! For compatibility between `std` and `no_std` environments,
//! re-export commonly used heap-allocated types: `Box`, `Rc` and `Vec`.

#[rustfmt::skip]
#[cfg(not(feature = "std"))]
pub use alloc::{boxed::Box, rc::Rc, vec::Vec};
#[rustfmt::skip]
#[cfg(feature = "std")]
pub use std::{boxed::Box, rc::Rc, vec::Vec};
