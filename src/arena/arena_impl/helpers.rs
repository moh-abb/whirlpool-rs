use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::error::ArenaError;
use crate::arena::error::ArenaResult;
use crate::arena::index::Index;

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
    #[allow(unused)]
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
}

fn slot_insert<T, U>(
    map: &mut impl IndexableMap<T>,
    index: Index<T>,
    value: T,
    result: U,
) -> ArenaResult<U> {
    let slot = map
        .get_mut_slot(index.clone())
        .ok_or(ArenaError::IndexOutOfBounds)?;
    match slot {
        Some(_) => Err(ArenaError::ExpectedFreeSlot),
        None => {
            let _ = slot.insert(value);
            Ok(result)
        }
    }
}

impl<T: ArenaItem, M: IndexableMap<T>> Arena<T> for IndexableMapArena<T, M> {
    fn size(&self) -> usize {
        let inner = self.0.borrow();
        inner.map.size()
    }

    fn alloc(&self, value: T) -> ArenaResult<Index<T>> {
        let mut inner = self.0.borrow_mut();
        let inner_next_index = inner.next_index;
        if inner_next_index == u16::MAX {
            // We can't progress to the next index
            return Err(ArenaError::LimitReached);
        }
        let index = Index::new(inner_next_index);
        let alloc_entry =
            slot_insert(&mut inner.map, index.clone(), value, index)?;
        inner.next_index += 1;
        Ok(alloc_entry)
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        let mut inner = self.0.borrow_mut();
        inner
            .map
            .get_mut_slot(index)
            .ok_or(ArenaError::IndexOutOfBounds)?
            .take()
            .map_or(Err(ArenaError::ExpectedFullSlot), Ok)
    }

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool> {
        let inner = self.0.borrow();
        inner
            .map
            .get_slot(index)
            .ok_or(ArenaError::IndexOutOfBounds)
            .map(Option::is_some)
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        let mut inner = self.0.borrow_mut();
        slot_insert(&mut inner.map, index, value, ())
    }

    fn inspect<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        let inner = self.0.borrow();
        inner
            .map
            .get_slot(index)
            .ok_or(ArenaError::IndexOutOfBounds)?
            .as_ref()
            .map_or(Err(ArenaError::ExpectedFullSlot), |x| Ok(func(x)))
    }

    fn inspect_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        let mut inner = self.0.borrow_mut();
        inner
            .map
            .get_mut_slot(index)
            .ok_or(ArenaError::IndexOutOfBounds)?
            .as_mut()
            .map_or(Err(ArenaError::ExpectedFullSlot), |x| Ok(func(x)))
    }
}
