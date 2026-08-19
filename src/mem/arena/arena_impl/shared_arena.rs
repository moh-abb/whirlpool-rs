use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

#[derive(Debug)]
pub struct SharedArena<'a, T, A>(RefCell<&'a mut A>, PhantomData<T>);

#[derive(Debug)]
pub struct SharedArenaRef<'r, 'a, T, A>(&'r SharedArena<'a, T, A>);

impl<'r, 'a, T, A> Clone for SharedArenaRef<'r, 'a, T, A> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'r, 'a, T, A> Copy for SharedArenaRef<'r, 'a, T, A> {}

impl<'a, T, A> SharedArena<'a, T, A>
where
    T: ArenaItem,
    A: Arena<T>,
{
    pub fn new(arena: &'a mut A) -> Self {
        Self(RefCell::new(arena), PhantomData)
    }

    pub fn make_ref<'r>(&'r self) -> SharedArenaRef<'r, 'a, T, A> {
        SharedArenaRef(self)
    }

    pub fn with_inner<U>(&self, func: impl FnOnce(&A) -> U) -> ArenaResult<U> {
        let borrowed = self
            .0
            .try_borrow()
            .map_err(|_| ArenaError::InvalidBorrow)?;
        Ok(func(borrowed.deref()))
    }

    pub fn with_inner_mut<U>(
        &self,
        func: impl FnOnce(&mut A) -> U,
    ) -> ArenaResult<U> {
        let mut borrowed = self
            .0
            .try_borrow_mut()
            .map_err(|_| ArenaError::InvalidBorrow)?;
        Ok(func(borrowed.deref_mut()))
    }
}

impl<'r, 'a, T, A> Arena<T> for SharedArenaRef<'r, 'a, T, A>
where
    T: ArenaItem,
    A: Arena<T>,
{
    fn size(&self) -> ArenaResult<usize> {
        self.0
            .with_inner_mut(|arena| arena.size())?
    }

    fn push(&mut self, value: T) -> ArenaResult<Index<T>> {
        self.0
            .with_inner_mut(|arena| arena.push(value))?
    }

    fn take(&mut self, index: Index<T>) -> ArenaResult<T> {
        self.0
            .with_inner_mut(|arena| arena.take(index))?
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        self.0
            .with_inner(|arena| arena.map(index, func))?
    }

    fn map_mut<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        self.0
            .with_inner_mut(|arena| arena.map_mut(index, func))?
    }
}
