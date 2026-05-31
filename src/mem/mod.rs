pub mod alloc_types;
pub mod arena;
pub mod clone;
pub mod cmp;
pub mod drop;
pub mod structures;

// Common re-exports

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
pub use structures::index::INVALID_INDEX_VALUE;
pub use structures::index::Index;
pub use structures::multiple::Multiple;

/// Asserts a unit [ArenaResult] is [Ok] when debug assertions are enabled.
pub fn debug_unwrap(result: ArenaResult<()>) {
    if cfg!(debug_assertions) {
        result.unwrap()
    }
}
