use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

impl<'a, T: ArenaItem, A: Arena<T>> Arena<T> for &'a mut A {
    fn size(&self) -> ArenaResult<usize> {
        A::size(*self)
    }

    fn push(&mut self, value: T) -> ArenaResult<Index<T>> {
        A::push(*self, value)
    }

    fn take(&mut self, index: Index<T>) -> ArenaResult<T> {
        A::take(*self, index)
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        A::map(*self, index, func)
    }

    fn map_mut<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        A::map_mut(*self, index, func)
    }
}
