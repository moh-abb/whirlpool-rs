use crate::arena::error::ArenaResult;
use crate::structures::index::Index;

pub mod arena_impl;
pub mod error;

pub trait ArenaItem: Ord + 'static {}
impl<T> ArenaItem for T where T: Ord + 'static {}

/// A trait to represent a simple arena, where items can be inserted, appended
/// (allocated), deleted (taken), and queried for occupied status based on an
/// [Index].
pub trait Arena<T: ArenaItem> {
    fn size(&self) -> usize;

    fn alloc(&self, value: T) -> ArenaResult<Index<T>>;

    fn take(&self, index: Index<T>) -> ArenaResult<T>;

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool>;

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()>;

    fn inspect<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U>;

    fn inspect_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U>;
}
