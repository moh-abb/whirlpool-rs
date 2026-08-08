pub mod alloc_types;
pub mod arena;
pub mod clone;
pub mod cmp;
pub mod drop;
pub mod linked;

#[macro_use]
pub mod structures;

// Common re-exports

use core::fmt::Debug;

pub use alloc_types::*;
pub use arena::Arena;
pub use arena::ArenaItem;
pub use arena::arena_impl::fixed_arena::FixedArena;
pub use arena::arena_impl::growable_arena::GrowableArena;
#[cfg(test)]
pub use arena::arena_impl::mock_arena::ArenaMocker;
pub use arena::arena_impl::scapegoat_arena::ScapegoatArena;
pub use arena::error::ArenaError;
pub use arena::error::ArenaResult;
pub use structures::chain::Chain;
pub use structures::index::Index;
pub use structures::multiple::Multiple;

/// Asserts a unit [Result] is [Ok] when debug assertions are enabled.
pub fn debug_unwrap<Err: Debug>(result: Result<(), Err>) {
    if cfg!(debug_assertions) {
        result.unwrap()
    }
}
