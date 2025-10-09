//! A program for creating Strudel live-coded music, specialised for embedded
//! systems with limited memory allocation.

#![cfg_attr(not(any(doc, feature = "std", test)), no_std)]
#![allow(clippy::let_and_return, rustdoc::private_intra_doc_links)]
#![deny(
    missing_docs,
    clippy::undocumented_unsafe_blocks,
    dead_code,
    unused,
    rustdoc::broken_intra_doc_links
)]

/// Used by [alloc_types].
#[allow(unused_extern_crates)]
extern crate alloc;

mod alloc_types;
mod arena;
mod ast;
