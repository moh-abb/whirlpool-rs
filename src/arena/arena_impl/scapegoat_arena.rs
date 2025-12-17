use core::cell::RefCell;
use core::ops::DerefMut;

use scapegoat::SgMap;
use scapegoat::map_types::Entry as SgEntry;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::error::ArenaError;
use crate::arena::error::ArenaResult;
use crate::arena::index::Index;

pub struct ScapegoatArena<T: ArenaItem, const N: usize>(RefCell<SgInner<T, N>>);

struct SgInner<T: ArenaItem, const N: usize> {
    next_index: u16,
    map: SgMap<Option<Index<T>>, Option<T>, N>,
}

impl<T: ArenaItem, const N: usize> ScapegoatArena<T, N> {
    #[allow(unused)]
    pub fn new() -> Self {
        // Note that we are allowed to make an arena larger than u16::MAX slots
        // (but we will never be able to allocate into the excess portion).
        let inner = SgInner { next_index: 0, map: SgMap::new() };
        Self(RefCell::new(inner))
    }

    #[allow(unused)]
    pub fn reset(&self) {
        self.with_inner(|inner| {
            inner.next_index = 0;
            inner.map.clear();
        })
    }

    fn with_inner<U>(&self, f: impl FnOnce(&mut SgInner<T, N>) -> U) -> U {
        let mut inner = self.0.borrow_mut();
        let inner_mut = inner.deref_mut();
        f(inner_mut)
    }
}

impl<T: ArenaItem, const N: usize> Arena<T> for ScapegoatArena<T, N> {
    fn alloc(&self, value: T) -> ArenaResult<Index<T>> {
        self.with_inner(|inner| {
            if inner.next_index == u16::MAX {
                // We can't progress to the next index
                return Err(ArenaError::LimitReached);
            }
            let ind = Index::new(inner.next_index);
            let entry = inner.map.entry(Some(ind.clone()));
            match entry {
                SgEntry::Occupied(_) => Err(ArenaError::ExpectedFreeSlot),
                SgEntry::Vacant(e) => {
                    e.insert(Some(value));
                    inner.next_index += 1;
                    Ok(ind)
                }
            }
        })
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        self.with_inner(|inner| {
            inner
                .map
                .remove(&Some(index))
                .flatten()
                .ok_or(ArenaError::ExpectedFullSlot)
        })
    }

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool> {
        self.with_inner(|inner| {
            inner
                .map
                .get(&Some(index))
                .map(Option::is_some)
                .ok_or(ArenaError::IndexOutOfBounds)
        })
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        self.with_inner(|inner| {
            let entry = inner.map.entry(Some(index));
            match entry {
                SgEntry::Occupied(_) => Err(ArenaError::ExpectedFreeSlot),
                SgEntry::Vacant(e) => {
                    e.insert(Some(value));
                    Ok(())
                }
            }
        })
    }
}
