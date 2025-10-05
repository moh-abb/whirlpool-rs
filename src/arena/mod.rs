pub mod index;
pub mod chain;

use index::Index;

pub trait ArenaItem: 'static {}
impl<T> ArenaItem for T where T: 'static {}

/// A trait to represent a simple arena, where items can be inserted, appended
/// (allocated), deleted (taken), and queried for occupied status based on an
/// [Index].
#[allow(dead_code)]
pub trait Arena<T: ArenaItem> {
    fn alloc(&self, value: T) -> Index<T>;

    fn take(&self, index: Index<T>) -> T;

    fn has_slot(&self, index: Index<T>) -> bool;

    fn insert(&self, index: Index<T>, value: T);
}
