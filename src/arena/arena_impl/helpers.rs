use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::error::ArenaError;
use crate::arena::error::ArenaResult;
use crate::structures::index::Index;

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

    /// Clears the map, dropping all items stored inside.
    fn clear(&mut self);
}

/// An adapter for implementing [Arena] with a backing [IndexableMap] field.
#[derive(Debug)]
pub struct IndexableMapArena<T, M>(RefCell<IMInner<T, M>>);

#[derive(Debug)]
struct IMInner<T, M> {
    next_index: u16,
    map: M,
    phantom: PhantomData<T>,
}

impl<T, M: IndexableMap<T>> IndexableMapArena<T, M> {
    /// Creates an arena with the given [IndexableMap] backing field.
    pub fn new(map: M) -> Self {
        Self(RefCell::new(IMInner { next_index: 0, map, phantom: PhantomData }))
    }

    /// Performs the given mutable action,
    /// using the arena's next index and map fields.
    pub fn with_inner_mut<U>(
        &self,
        f: impl FnOnce(&mut u16, &mut M) -> U,
    ) -> U {
        let mut inner = self.0.borrow_mut();
        let IMInner { next_index, map, phantom: _ } = inner.deref_mut();
        f(next_index, map)
    }

    fn with_mut_slot<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut Option<T>) -> U,
    ) -> ArenaResult<U> {
        let mut inner = self.0.borrow_mut();
        inner
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
        let inner = self.0.borrow();
        inner
            .map
            .get_slot(index)
            .map(func)
            .ok_or(ArenaError::IndexOutOfBounds)
    }
}

impl<T: ArenaItem, M: IndexableMap<T>> Arena<T> for IndexableMapArena<T, M> {
    fn size(&self) -> usize {
        let inner = self.0.borrow();
        inner.map.size()
    }

    fn alloc(&self, value: T) -> ArenaResult<Index<T>> {
        // Stage 1. Check for the arena limit being reached.
        let inner_ref = self.0.borrow();
        let inner_next_index = inner_ref.next_index;
        if inner_next_index == u16::MAX {
            // We can't progress to the next index
            return Err(ArenaError::LimitReached);
        }
        let index = Index::new(inner_next_index);
        core::mem::drop(inner_ref);
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
        let mut inner_mut = self.0.borrow_mut();
        inner_mut.next_index += 1;
        alloc_entry
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        self.with_mut_slot(index, Option::take)?
            .ok_or(ArenaError::ExpectedFullSlot)
    }

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool> {
        self.with_slot(index, Option::is_some)
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        self.with_mut_slot(index, |slot| {
            let _ = slot.insert(value);
        })
    }

    fn inspect<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        self.with_slot(index, |slot| slot.as_ref().map(func))?
            .ok_or(ArenaError::ExpectedFullSlot)
    }

    fn inspect_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        self.with_mut_slot(index, |slot| slot.as_mut().map(func))?
            .ok_or(ArenaError::ExpectedFullSlot)
    }
}
