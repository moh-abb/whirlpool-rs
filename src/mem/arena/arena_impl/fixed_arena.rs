#![cfg(test)]

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::BTreeMap;
use crate::mem::Index;
use crate::mem::arena::arena_impl::helpers::IndexableMap;
use crate::mem::arena::arena_impl::helpers::IndexableMapArena;

/// An [Arena] for which allocation can temporarily be allowed and disallowed.
#[derive(Debug)]
pub struct FixableArena<T: ArenaItem> {
    mutable: bool,
    arena: IndexableMapArena<T, BTreeMapAdapter<T>>,
}

impl<T: ArenaItem> Default for FixableArena<T> {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug)]
struct BTreeMapAdapter<T>(BTreeMap<Index<T>, Option<T>>);

impl<T: ArenaItem> IndexableMap<T> for BTreeMapAdapter<T> {
    fn size(&self) -> usize {
        self.0.len()
    }

    fn get_slot(&self, index: Index<T>) -> Option<&Option<T>> {
        self.0.get(&index)
    }

    fn get_mut_slot(&mut self, index: Index<T>) -> Option<&mut Option<T>> {
        self.0.get_mut(&index)
    }
}

impl<T: ArenaItem> FixableArena<T> {
    /// Creates a new [FixableArena], initially fixed.
    #[allow(unused)]
    pub fn new(entries: impl IntoIterator<Item = (Index<T>, T)>) -> Self {
        // Note that we are allowed to make an arena larger than u16::MAX slots
        // (but we will never be able to allocate into the excess portion).
        let btree_map = BTreeMap::from_iter(
            entries
                .into_iter()
                .map(|(k, v)| (k, Some(v))),
        );
        let entry_to_index = |key_pair: (&Index<T>, &Option<T>)| {
            let (index, _) = key_pair;
            usize::from(index.clone()) + 1
        };
        let mut start_index = u16::try_from(
            btree_map
                .last_key_value()
                .map_or(0, entry_to_index),
        )
        .unwrap();
        let mut arena = IndexableMapArena::new(BTreeMapAdapter(btree_map));
        arena
            .with_inner_mut(|next_index, _| {
                *next_index = start_index;
            })
            .unwrap();
        Self { mutable: false, arena }
    }

    #[allow(unused)]
    pub fn allow_alloc(&mut self) {
        self.mutable = true;
    }

    #[allow(unused)]
    pub fn forbid_alloc(&mut self) {
        self.mutable = false;
    }
}

impl<T: ArenaItem> Arena<T> for FixableArena<T> {
    fn size(&self) -> usize {
        self.arena.size()
    }

    fn push(&self, value: T) -> ArenaResult<Index<T>> {
        if !self.mutable {
            panic!(
                "Should not call [Arena::push] on FixableArena while immutable"
            )
        } else {
            // We need to extend the inner map with one extra slot, provided we
            // have not already exceeded the limit.
            self.arena
                .with_inner_mut(|next_index, map| {
                    if *next_index == u16::MAX {
                        return Err(ArenaError::LimitReached);
                    }
                    map.0
                        .insert(Index::new(*next_index), None);
                    Ok(())
                })??;
            self.arena.push(value)
        }
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        if !self.mutable {
            panic!(
                "Should not call [Arena::take] on FixableArena while immutable"
            )
        } else {
            self.arena.take(index)
        }
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        self.arena.map(index, func)
    }

    fn map_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        if !self.mutable {
            unreachable!(
                "Should not call [Arena::inspect_mut] on FixableArena while immutable"
            )
        } else {
            self.arena.map_mut(index, func)
        }
    }
}
