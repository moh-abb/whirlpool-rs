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
}

impl<T: ArenaItem, const N: usize> Arena<T> for ScapegoatArena<T, N> {
    fn size(&self) -> ArenaResult<usize> {
        self.0.size()
    }

    fn push(&mut self, value: T) -> ArenaResult<Index<T>> {
        self.0.push(value)
    }

    fn take(&mut self, index: Index<T>) -> ArenaResult<T> {
        self.0.take(index)
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        self.0.map(index, func)
    }

    fn map_mut<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        self.0.map_mut(index, func)
    }
}
