use crate::alloc_types::Vec;
use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::arena_impl::helpers::IndexableMap;
use crate::arena::arena_impl::helpers::IndexableMapArena;
use crate::arena::error::ArenaError;
use crate::arena::error::ArenaResult;
use crate::arena::index::Index;

#[derive(Debug)]
pub struct GrowableArena<T: ArenaItem>(IndexableMapArena<T, GAMap<T>>);

#[derive(Debug)]
struct GAMap<T>(Vec<Option<T>>);

impl<T> IndexableMap<T> for GAMap<T> {
    fn get_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>> {
        self.0.get_mut(usize::from(index))
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl<T: ArenaItem> GrowableArena<T> {
    #[allow(unused)]
    pub fn new() -> Self {
        // Note that we are allowed to make an arena larger than u16::MAX slots
        // (but we will never be able to allocate into the excess portion).
        Self(IndexableMapArena::new(GAMap(Vec::new())))
    }

    #[allow(unused)]
    pub fn reset(&self) {
        self.0.with_inner(|next_index, map| {
            *next_index = 0;
            map.clear();
        })
    }
}

impl<T: ArenaItem> Arena<T> for GrowableArena<T> {
    fn alloc(&self, value: T) -> ArenaResult<Index<T>> {
        // We need to extend the inner `Vec` with one extra slot, provided we
        // have not already exceeded the limit.
        self.0.with_inner(|next_index, map| {
            if *next_index == u16::MAX {
                return Err(ArenaError::LimitReached);
            }
            map.0.push(None);
            Ok(())
        })?;
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
