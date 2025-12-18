use scapegoat::SgMap;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::arena_impl::helpers::IndexableMap;
use crate::arena::arena_impl::helpers::IndexableMapArena;
use crate::arena::error::ArenaResult;
use crate::arena::index::Index;

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

    fn get_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>> {
        self.0.get_mut(&Some(index))
    }

    fn clear(&mut self) {
        self.0.clear()
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
        self.0.with_inner(|next_index, map| {
            *next_index = 0;
            map.clear();
        })
    }
}

impl<T: ArenaItem, const N: usize> Arena<T> for ScapegoatArena<T, N> {
    fn size(&self) -> usize {
        self.0.size()
    }

    fn alloc(&self, value: T) -> ArenaResult<Index<T>> {
        self.0.alloc(value)
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        self.0.take(index)
    }

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool> {
        self.0.has_slot(index)
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        self.0.insert(index, value)
    }
}
