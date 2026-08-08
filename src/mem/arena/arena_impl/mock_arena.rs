#![cfg(test)]

use core::cell::RefCell;
use core::fmt::Debug;
use core::ops::Deref;
use core::ops::DerefMut;

use spin::RwLockReadGuard;
use spin::RwLockWriteGuard;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

#[mockall::automock(
    type Ref<'a> = RwLockReadGuard<'a,  Option<T>>;
    type Mut<'a> = RwLockWriteGuard<'a, Option< T>>;
)]
pub trait ArenaAdapter<T: ArenaItem>: Debug {
    type Ref<'a>: Deref<Target = Option<T>>
    where
        Self: 'a;
    type Mut<'a>: DerefMut<Target = Option<T>>
    where
        Self: 'a;

    fn size(&self) -> usize;

    fn alloc(&mut self, value: T) -> ArenaResult<Index<T>>;

    fn get_slot<'a>(&'a self, index: Index<T>) -> ArenaResult<Self::Ref<'a>>;

    fn get_mut_slot<'a>(
        &'a mut self,
        index: Index<T>,
    ) -> ArenaResult<Self::Mut<'a>>;
}

#[derive(Debug)]
pub struct ArenaMocker<T: ArenaItem>(RefCell<MockArenaAdapter<T>>);

impl<T: ArenaItem> Default for ArenaMocker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ArenaItem> ArenaMocker<T> {
    pub fn new() -> Self {
        Self(RefCell::new(MockArenaAdapter::new()))
    }

    pub fn borrow_mut(&self) -> impl DerefMut<Target = MockArenaAdapter<T>> {
        self.0.borrow_mut()
    }
}

impl<T: ArenaItem + Clone> Arena<T> for ArenaMocker<T> {
    fn size(&self) -> usize {
        self.0.borrow().size()
    }

    fn push(&self, value: T) -> ArenaResult<Index<T>> {
        self.0.borrow_mut().alloc(value)
    }

    fn take(&self, index: Index<T>) -> ArenaResult<T> {
        let mut inner = self.0.borrow_mut();
        let mut slot = inner.get_mut_slot(index)?;
        Ok(slot.take().unwrap())
    }

    fn insert(&self, index: Index<T>, value: T) -> ArenaResult<()> {
        let mut inner = self.0.borrow_mut();
        let mut slot = inner.get_mut_slot(index)?;
        let _ = slot.insert(value);
        Ok(())
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        let inner = self.0.borrow();
        let slot = inner.get_slot(index)?;
        Ok(func(slot.as_ref().unwrap()))
    }

    fn map_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        let mut inner = self.0.borrow_mut();
        let mut slot = inner.get_mut_slot(index)?;
        Ok(func(slot.as_mut().unwrap()))
    }
}
