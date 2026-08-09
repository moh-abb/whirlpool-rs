use core::marker::PhantomData;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::Index;

/// An arena-based Stack that is implemented via a singly-linked-list structure.
pub struct Stack<'a, T, A> {
    last_item: Option<Index<StackChain<T>>>,
    size: usize,
    arena: &'a mut A,
}

/// The error type when pushing/popping/accessing a stack item.
#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackError {
    ArenaErr(ArenaError),
}

pub type StackResult<T> = Result<T, StackError>;

/// Stores one unit of the linked item in a stack.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackChain<T> {
    item: T,
    prev: Option<Index<Self>>,
}

impl<'a, T, A> Stack<'a, T, A> {
    pub const fn new(arena: &'a mut A) -> Self {
        Self { last_item: None, size: 0, arena }
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn is_empty(&self) -> bool {
        self.last_item.is_none()
    }
}

impl<'a, T, A> Stack<'a, T, A>
where
    T: ArenaItem,
    A: Arena<StackChain<T>>,
{
    pub fn push(&mut self, arena: &mut A, item: T) -> StackResult<()> {
        let index =
            arena.push(StackChain { item, prev: self.last_item.take() })?;
        self.last_item = Some(index);
        self.size = self.size.wrapping_add(1);
        Ok(())
    }

    pub fn pop(&mut self, arena: &mut A) -> StackResult<Option<T>> {
        let index = match self.last_item.take() {
            Some(index) => index,
            None => return Ok(None),
        };

        let StackChain { item, prev } = arena.take(index)?;
        self.last_item = prev;
        self.size = self.size.wrapping_sub(1);

        Ok(Some(item))
    }

    pub fn map<U>(
        &self,
        arena: &A,
        func: impl FnOnce(&T) -> U,
    ) -> StackResult<Option<U>> {
        let index = match self.last_item.clone() {
            Some(index) => index,
            None => return Ok(None),
        };

        let res =
            arena.map(index, |StackChain { item, prev: _ }| func(item))?;

        Ok(Some(res))
    }

    pub fn map_mut<U>(
        &mut self,
        arena: &A,
        func: impl FnOnce(&mut T) -> U,
    ) -> StackResult<Option<U>> {
        let index = match self.last_item.clone() {
            Some(index) => index,
            None => return Ok(None),
        };

        let res =
            arena.map_mut(index, |StackChain { item, prev: _ }| func(item))?;

        Ok(Some(res))
    }
}

impl<'a, T, A> Stack<'a, T, A>
where
    T: ArenaItem + Clone,
    A: Arena<StackChain<T>>,
{
    pub fn clone_last(&self, arena: &A) -> StackResult<Option<T>> {
        self.map(arena, Clone::clone)
    }
}
