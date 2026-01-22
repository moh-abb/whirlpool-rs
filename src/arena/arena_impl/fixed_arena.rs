use crate::alloc_types::BTreeMap;
use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::arena_impl::helpers::IndexableMap;
use crate::arena::arena_impl::helpers::IndexableMapArena;
use crate::arena::error::ArenaResult;
use crate::arena::index::Index;

#[derive(Debug)]
pub struct FixedArena<T: ArenaItem>(IndexableMapArena<T, BTreeMapAdapter<T>>);

impl<T: ArenaItem> Default for FixedArena<T> {
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

    fn get_mut_slot(&mut self, _index: Index<T>) -> Option<&mut Option<T>> {
        unreachable!(
            "Should not call [IndexableMap::get_mut_slot] on FixedArena"
        )
    }

    fn clear(&mut self) {
        unreachable!("Should not call [IndexableMap::clear] on FixedArena")
    }
}

impl<T: ArenaItem> FixedArena<T> {
    #[allow(unused)]
    pub fn new(entries: impl IntoIterator<Item = (Index<T>, T)>) -> Self {
        // Note that we are allowed to make an arena larger than u16::MAX slots
        // (but we will never be able to allocate into the excess portion).
        let btree_map = BTreeMap::from_iter(
            entries
                .into_iter()
                .map(|(k, v)| (k, Some(v))),
        );
        Self(IndexableMapArena::new(BTreeMapAdapter(btree_map)))
    }
}

impl<T: ArenaItem> Arena<T> for FixedArena<T> {
    fn size(&self) -> usize {
        self.0.size()
    }

    fn alloc(&self, _: T) -> ArenaResult<Index<T>> {
        unreachable!("Should not call [Arena::alloc] on FixedArena")
    }

    fn take(&self, _: Index<T>) -> ArenaResult<T> {
        unreachable!("Should not call [Arena::take] on FixedArena")
    }

    fn has_slot(&self, index: Index<T>) -> ArenaResult<bool> {
        self.0.has_slot(index)
    }

    fn insert(&self, _: Index<T>, _: T) -> ArenaResult<()> {
        unreachable!("Should not call [Arena::insert] on FixedArena")
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
        _: Index<T>,
        _: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        unreachable!("Should not call [Arena::inspect_mut] on FixedArena")
    }
}
