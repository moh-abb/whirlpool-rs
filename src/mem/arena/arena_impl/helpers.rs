use core::marker::PhantomData;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

pub trait IndexableMap<T> {
    /// Returns the number of full slots in the current map.
    fn size(&self) -> usize;

    /// Provides an immutable entry of the map at the given index.
    ///
    /// Returns `None` if the index is out of bounds, otherwise returns
    /// `Some(slot)` where `slot` is an immutable value mapped by `index`.
    fn get_slot(&self, index: Index<T>) -> Option<&Option<T>>;

    /// Provides a mutable entry of the map at the given index.
    ///
    /// Returns `None` if the index is out of bounds, otherwise returns
    /// `Some(slot)` where `slot` is a mutable value mapped by `index`.
    fn get_mut_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>>;
}

/// An adapter for implementing [Arena] with a backing [IndexableMap] field.
#[derive(Debug)]
pub struct IndexableMapArena<T, M>(IMInner<T, M>);

#[derive(Debug)]
struct IMInner<T, M> {
    next_index: u16,
    map: M,
    phantom: PhantomData<T>,
}

impl<T, M: IndexableMap<T>> IndexableMapArena<T, M> {
    /// Creates an arena with the given [IndexableMap] backing field.
    pub fn new(map: M) -> Self {
        Self(IMInner { next_index: 0, map, phantom: PhantomData })
    }

    /// Performs the given mutable action,
    /// using the arena's next index and map fields.
    #[allow(unused)]
    #[must_use]
    pub fn with_inner_mut<U>(
        &mut self,
        f: impl FnOnce(&mut u16, &mut M) -> U,
    ) -> ArenaResult<U> {
        Ok(f(&mut self.0.next_index, &mut self.0.map))
    }

    fn with_mut_slot<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut Option<T>) -> U,
    ) -> ArenaResult<U> {
        self.0
            .map
            .get_mut_slot(index)
            .map(func)
            .ok_or(ArenaError::IndexOutOfBounds)
    }

    fn with_slot<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&Option<T>) -> U,
    ) -> ArenaResult<U> {
        self.0
            .map
            .get_slot(index)
            .map(func)
            .ok_or(ArenaError::IndexOutOfBounds)
    }
}

impl<T: ArenaItem, M: IndexableMap<T>> Arena<T> for IndexableMapArena<T, M> {
    fn size(&self) -> ArenaResult<usize> {
        Ok(self.0.map.size())
    }

    fn push(&mut self, value: T) -> ArenaResult<Index<T>> {
        // Stage 1. Check for the arena limit being reached.
        let inner_next_index = self.0.next_index;
        if inner_next_index == u16::MAX {
            // We can't progress to the next index
            return Err(ArenaError::LimitReached);
        }
        let index = Index::new(inner_next_index);
        // Stage 2. Obtain a slot at the given index and
        let alloc_entry =
            self.with_mut_slot(index.clone(), |slot| match slot {
                Some(_) => Err(ArenaError::ExpectedFreeSlot),
                None => {
                    let _ = slot.insert(value);
                    Ok(index)
                }
            })?;
        // Stage 3. Increment the next index.
        self.0.next_index += 1;
        alloc_entry
    }

    fn take(&mut self, index: Index<T>) -> ArenaResult<T> {
        self.with_mut_slot(index, Option::take)?
            .ok_or(ArenaError::ExpectedFullSlot)
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        self.with_slot(index, |slot| slot.as_ref().map(func))?
            .ok_or(ArenaError::ExpectedFullSlot)
    }

    fn map_mut<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        self.with_mut_slot(index, |slot| slot.as_mut().map(func))?
            .ok_or(ArenaError::ExpectedFullSlot)
    }
}
