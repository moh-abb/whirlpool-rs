pub mod arena_impl;
pub mod chain;
pub mod chain_iter;
pub mod equality;
pub mod error;
pub mod extension;
pub mod handler;
pub mod index;
pub mod tuple;
mod tuple_macros;

use error::ArenaResult;
use index::Index;

pub trait ArenaItem: Ord + 'static {}
impl<T> ArenaItem for T where T: Ord + 'static {}

/// A trait to represent a simple arena, where items can be inserted, appended
/// (allocated), deleted (taken), and queried for occupied status based on an
/// [Index].
#[allow(dead_code)]
pub trait Arena<T: ArenaItem> {
    fn alloc(&self, value: T) -> ArenaResult<Index<T>>;

    fn take(&self, index: Index<T>) -> ArenaResult<T>;

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool>;

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()>;
}
