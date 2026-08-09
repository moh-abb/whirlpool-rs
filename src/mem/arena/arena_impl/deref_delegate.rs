use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

impl<'a, T: ArenaItem, A: Arena<T>> Arena<T> for &'a mut A {
    fn size(&self) -> usize {
        A::size(*self)
    }

    fn push(&self, value: T) -> ArenaResult<Index<T>> {
        A::push(*self, value)
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        A::take(*self, index)
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        A::insert(*self, index, value)
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        A::map(*self, index, func)
    }

    fn map_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        A::map_mut(*self, index, func)
    }
}
