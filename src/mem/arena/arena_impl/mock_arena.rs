#![cfg(test)]

use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt::Debug;
use core::ops::DerefMut;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

type Slot<T> = Rc<RefCell<Option<T>>>;

#[mockall::automock]
pub trait ArenaAdapter<T: ArenaItem>: Debug {
    fn size(&self) -> usize;

    fn push(&mut self, value: T) -> ArenaResult<Index<T>>;

    fn get_slot(&self, index: Index<T>) -> ArenaResult<Slot<T>>;

    fn get_mut_slot<'a>(&mut self, index: Index<T>) -> ArenaResult<Slot<T>>;
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
    fn size(&self) -> ArenaResult<usize> {
        Ok(self.0.borrow().size())
    }

    fn push(&mut self, value: T) -> ArenaResult<Index<T>> {
        self.0.borrow_mut().push(value)
    }

    fn take(&mut self, index: Index<T>) -> ArenaResult<T> {
        let mut inner = self.0.borrow_mut();
        let slot = inner.get_mut_slot(index)?;
        Ok(slot.borrow_mut().take().unwrap())
    }

    fn map<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        let inner = self.0.borrow();
        let slot = inner.get_slot(index)?;
        Ok(func(slot.borrow().as_ref().unwrap()))
    }

    fn map_mut<U>(
        &mut self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        let mut inner = self.0.borrow_mut();
        let slot = inner.get_mut_slot(index)?;
        Ok(func(slot.borrow_mut().as_mut().unwrap()))
    }
}
