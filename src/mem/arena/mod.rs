use crate::mem::ArenaResult;
use crate::mem::Index;

pub mod arena_impl;
pub mod error;

pub trait ArenaItem: Ord {}
impl<T> ArenaItem for T where T: Ord {}

/// A trait to represent a simple arena, where items can be inserted, appended
/// (allocated), deleted (taken), and queried for occupied status based on an
/// [Index].
pub trait Arena<T: ArenaItem> {
    fn size(&self) -> ArenaResult<usize>;

    fn push(&mut self, value: T) -> ArenaResult<Index<T>>;

    fn take(&mut self, index: Index<T>) -> ArenaResult<T>;

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U>;

    fn map_mut<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U>;
}
