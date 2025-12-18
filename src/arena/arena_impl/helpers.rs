use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::DerefMut;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::error::ArenaError;
use crate::arena::error::ArenaResult;
use crate::arena::index::Index;

pub trait IndexableMap<T> {
    /// Provides the entry of the map at the given index.
    ///
    /// Returns `None` if the index is out of bounds, otherwise returns
    /// `Some(slot)` where `slot` is a mutable value mapped by `index`.
    fn get_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>>;

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

    /// Performs the given action using the arena's next index and map fields.
    pub fn with_inner<U>(&self, f: impl FnOnce(&mut u16, &mut M) -> U) -> U {
        let mut inner = self.0.borrow_mut();
        let IMInner { next_index, map, phantom: _ } = inner.deref_mut();
        f(next_index, map)
    }

    /// Indexes the arena's map at the given `index` (returning
    /// [ArenaError::IndexOutOfBounds] on failure), then performs the given
    /// `action` on the slot (i.e., `&mut Option<T>`), then modifies the arena's
    /// next index using `update_index`.
    fn with_slot<U>(
        &self,
        index: Index<T>,
        update_index: impl FnOnce(&mut u16),
        action: impl FnOnce(&mut Option<T>) -> ArenaResult<U>,
    ) -> ArenaResult<U> {
        self.with_inner(|next_index, map| {
            let entry = map
                .get_slot(index.clone())
                .ok_or(ArenaError::IndexOutOfBounds)?;
            let result = action(entry);
            update_index(next_index);
            result
        })
    }
}

impl<T: ArenaItem, M: IndexableMap<T>> Arena<T> for IndexableMapArena<T, M> {
    fn alloc(&self, value: T) -> ArenaResult<Index<T>> {
        let inner_next_index = self.with_inner(|next_index, _| *next_index);
        let index = Index::new(inner_next_index);
        if inner_next_index == u16::MAX {
            // We can't progress to the next index
            return Err(ArenaError::LimitReached);
        }
        self.with_slot(
            index.clone(),
            |ind| *ind += 1,
            |entry| match entry {
                Some(_) => Err(ArenaError::ExpectedFreeSlot),
                None => {
                    let _ = entry.insert(value);
                    Ok(index)
                }
            },
        )
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        let take_entry = |entry: &mut Option<T>| {
            entry
                .take()
                .ok_or(ArenaError::ExpectedFullSlot)
        };
        self.with_slot(index, |_| (), take_entry)
    }

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool> {
        self.with_slot(index, |_| (), |entry| Ok(entry.is_some()))
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        let insert_entry = |entry: &mut Option<T>| match entry {
            Some(_) => Err(ArenaError::ExpectedFreeSlot),
            None => {
                let _ = entry.insert(value);
                Ok(())
            }
        };
        self.with_slot(index, |_| (), insert_entry)
    }
}
