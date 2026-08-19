use core::cmp::Ordering;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Index;

/// An arena-based Stack that is implemented via a singly-linked-list structure.
pub struct Stack<'a, T, A> {
    last_item: Option<Index<StackChain<T>>>,
    size: u16,
    arena: &'a mut A,
}

/// The error type when pushing/popping/accessing a stack item.
#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackError {
    ArenaErr(ArenaError),
}

pub type StackResult<T> = Result<T, StackError>;

/// Stores one unit of the linked item in a stack.
#[derive(Debug)]
pub struct StackChain<T> {
    /// Used to uniquely identify the item in the arena.
    index: Index<Self>,
    item: T,
    prev: Option<Index<Self>>,
}

impl<T> PartialEq for StackChain<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index.eq(&other.index)
    }
}

impl<T> Eq for StackChain<T> {}

impl<T> PartialOrd for StackChain<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for StackChain<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<'a, T, A> Stack<'a, T, A> {
    pub const fn new(arena: &'a mut A) -> Self {
        Self { last_item: None, size: 0, arena }
    }

    pub const fn size(&self) -> u16 {
        self.size
    }

    pub const fn is_empty(&self) -> bool {
        self.last_item.is_none()
    }
}

impl<'a, T, A> Stack<'a, T, A>
where
    A: Arena<StackChain<T>>,
{
    pub fn push(&mut self, item: T) -> StackResult<()> {
        let index = self.arena.push(StackChain {
            item,
            prev: self.last_item.take(),
            index: Index::new_invalid(),
        })?;
        self.last_item = Some(index.clone());
        // Set the index in the arena to point to itself.
        let update_chain = |stack_chain: &mut StackChain<T>| {
            stack_chain.index = index.clone();
        };
        self.arena
            .map_mut(index.clone(), update_chain)?;
        self.size = self.size.wrapping_add(1);

        Ok(())
    }

    pub fn pop(&mut self) -> StackResult<Option<T>> {
        let index = match self.last_item.take() {
            Some(index) => index,
            None => return Ok(None),
        };

        let StackChain { item, prev, .. } = self.arena.take(index)?;
        self.last_item = prev;
        self.size = self.size.wrapping_sub(1);

        Ok(Some(item))
    }

    pub fn clear(&mut self) -> StackResult<()> {
        while let Some(_) = self.pop()? {
            // Loop.
        }
        Ok(())
    }

    pub fn map<U>(&self, func: impl FnOnce(&T) -> U) -> StackResult<Option<U>> {
        let index = match self.last_item.clone() {
            Some(index) => index,
            None => return Ok(None),
        };

        let res = self
            .arena
            .map(index, |StackChain { item, .. }| func(item))?;

        Ok(Some(res))
    }

    pub fn map_mut<U>(
        &mut self,
        func: impl FnOnce(&mut T) -> U,
    ) -> StackResult<Option<U>> {
        let index = match self.last_item.clone() {
            Some(index) => index,
            None => return Ok(None),
        };

        let res = self
            .arena
            .map_mut(index, |StackChain { item, .. }| func(item))?;

        Ok(Some(res))
    }
}

impl<'a, T, A> Stack<'a, T, A>
where
    T: Clone,
    A: Arena<StackChain<T>>,
{
    pub fn clone_last(&self) -> StackResult<Option<T>> {
        self.map(Clone::clone)
    }
}
