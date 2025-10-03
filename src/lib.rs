//! A program for creating Strudel live-coded music, specialised for embedded
//! systems with limited memory allocation.

#![cfg_attr(not(any(doc, feature = "std", test)), no_std)]
#![allow(clippy::let_and_return)]
#![deny(missing_docs, clippy::undocumented_unsafe_blocks, dead_code)]

mod arena;