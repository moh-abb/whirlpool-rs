use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Vec;
use crate::mem::arena::arena_impl::helpers::IndexableMap;
use crate::mem::arena::arena_impl::helpers::IndexableMapArena;

#[derive(Debug)]
pub struct GrowableArena<T: ArenaItem>(IndexableMapArena<T, GAMap<T>>);

#[derive(Debug)]
struct GAMap<T>(Vec<Option<T>>);

impl<T> IndexableMap<T> for GAMap<T> {
    fn size(&self) -> usize {
        self.0
            .iter()
            .filter_map(Option::as_ref)
            .count()
    }

    fn get_slot(&self, index: Index<T>) -> Option<&Option<T>> {
        self.0.get(usize::from(index))
    }

    fn get_mut_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>> {
        self.0.get_mut(usize::from(index))
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl<T: ArenaItem> Default for GrowableArena<T> {
    fn default() -> Self {
        Self::new()
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
        self.0
            .with_inner_mut(|next_index, map| {
                *next_index = 0;
                map.clear();
            })
    }
}

impl<T: ArenaItem> Arena<T> for GrowableArena<T> {
    fn size(&self) -> usize {
        self.0.size()
    }

    fn push(&self, value: T) -> ArenaResult<Index<T>> {
        // We need to extend the inner `Vec` with one extra slot, provided we
        // have not already exceeded the limit.
        self.0
            .with_inner_mut(|next_index, map| {
                if *next_index == u16::MAX {
                    return Err(ArenaError::LimitReached);
                }
                map.0.push(None);
                Ok(())
            })?;
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
