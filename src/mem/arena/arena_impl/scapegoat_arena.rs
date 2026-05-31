use scapegoat::SgMap;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::arena::arena_impl::helpers::IndexableMap;
use crate::mem::arena::arena_impl::helpers::IndexableMapArena;

/// An [Arena] that uses [scapegoat]'s backing structures for allocating
/// structures without dynamic allocation.
#[derive(Debug)]
pub struct ScapegoatArena<T: ArenaItem, const N: usize>(
    IndexableMapArena<T, SgInnerMap<T, N>>,
);

#[derive(Debug)]
struct SgInnerMap<T: ArenaItem, const N: usize>(
    SgMap<Option<Index<T>>, Option<T>, N>,
);

impl<T: ArenaItem, const N: usize> IndexableMap<T> for SgInnerMap<T, N> {
    fn size(&self) -> usize {
        self.0.len()
    }

    fn get_slot(&self, index: Index<T>) -> Option<&Option<T>> {
        self.0.get(&Some(index))
    }

    fn get_mut_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>> {
        self.0.get_mut(&Some(index))
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl<T: ArenaItem, const N: usize> Default for ScapegoatArena<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ArenaItem, const N: usize> ScapegoatArena<T, N> {
    #[allow(unused)]
    pub fn new() -> Self {
        // Note that we are allowed to make an arena larger than u16::MAX slots
        // (but we will never be able to allocate into the excess portion).
        Self(IndexableMapArena::new(SgInnerMap(SgMap::new())))
    }

    #[allow(unused)]
    pub fn reset(&self) {
        self.0
            .with_inner_mut(|next_index, map| {
                *next_index = 0;
                map.clear();
            })
    }
}

impl<T: ArenaItem, const N: usize> Arena<T> for ScapegoatArena<T, N> {
    fn size(&self) -> usize {
        self.0.size()
    }

    fn push(&self, value: T) -> ArenaResult<Index<T>> {
        self.0.push(value)
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        self.0.take(index)
    }

    fn inspect<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        self.0.inspect(index, func)
    }

    fn inspect_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        self.0.inspect_mut(index, func)
    }
}
