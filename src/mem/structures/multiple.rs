use core::marker::PhantomData;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::Index;

type StartEnd<Item> = Option<(Index<Chain<Item>>, Index<Chain<Item>>)>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Multiple<Item: ArenaItem> {
    length: u16,
    start_end: StartEnd<Item>,
}

impl<Item: ArenaItem> Multiple<Item> {
    pub fn new_nonempty(
        length: u16,
        start: Index<Chain<Item>>,
        end: Index<Chain<Item>>,
    ) -> Self {
        Self { length, start_end: Some((start, end)) }
    }

    pub fn new_empty() -> Self {
        Self { length: 0, start_end: None }
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn start_end(&self) -> StartEnd<Item> {
        self.start_end.clone()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.start_end.is_none(), self.length == 0);
        self.length == 0
    }

    pub fn push_front(
        &mut self,
        arena: &impl Arena<Chain<Item>>,
        elem_index: Index<Chain<Item>>,
    ) {
        // The element should not be connected to anything.
        let elem_is_singleton = arena
            .map(elem_index.clone(), |e| {
                e.clone().pop_prev().is_none() && e.clone().pop_next().is_none()
            })
            .unwrap();
        debug_assert!(elem_is_singleton);
        let new_start_end = if self.is_empty() {
            // The list is empty; update the chain to a singleton.
            (elem_index.clone(), elem_index.clone())
        } else {
            // The list is not empty; push to the end.
            let (start, end) = self.start_end.clone().unwrap();
            let new_start = elem_index.clone();
            let link_start_to_new_start = |start_elem: &mut Chain<Item>| {
                start_elem.set_prev(new_start.clone());
            };
            let link_new_start_to_start = |new_start_elem: &mut Chain<Item>| {
                new_start_elem.set_next(start.clone());
            };
            arena
                .map_mut(start.clone(), link_start_to_new_start)
                .unwrap();
            arena
                .map_mut(new_start.clone(), link_new_start_to_start)
                .unwrap();
            (new_start, end)
        };
        // Update the start and end indices, and the length
        self.length += 1;
        let _ = self.start_end.insert(new_start_end);
    }

    pub fn push_back(
        &mut self,
        arena: &impl Arena<Chain<Item>>,
        elem_index: Index<Chain<Item>>,
    ) {
        // The element should not be connected to anything.
        let elem_is_singleton = arena
            .map(elem_index.clone(), |e| {
                e.clone().pop_prev().is_none() && e.clone().pop_next().is_none()
            })
            .unwrap();
        debug_assert!(elem_is_singleton);
        let new_start_end = if self.is_empty() {
            // The list is empty; update the chain to a singleton.
            (elem_index.clone(), elem_index.clone())
        } else {
            // The list is not empty; push to the end.
            let (start, end) = self.start_end.clone().unwrap();
            let new_end = elem_index.clone();
            let link_end_to_new_end = |end_elem: &mut Chain<Item>| {
                end_elem.set_next(new_end.clone());
            };
            let link_new_end_to_end = |new_end_elem: &mut Chain<Item>| {
                new_end_elem.set_prev(end.clone());
            };
            arena
                .map_mut(end.clone(), link_end_to_new_end)
                .unwrap();
            arena
                .map_mut(new_end.clone(), link_new_end_to_end)
                .unwrap();
            (start, new_end)
        };
        // Update the start and end indices, and the length
        self.length += 1;
        let _ = self.start_end.insert(new_start_end);
    }

    pub fn pop_back(
        &mut self,
        arena: &impl Arena<Chain<Item>>,
    ) -> Option<Index<Chain<Item>>> {
        let Some((start, end)) = self.start_end.clone() else {
            debug_assert_eq!(self.length, 0);
            return None;
        };
        let end_prev = arena
            .map_mut(end.clone(), |end_elem| {
                let end_next = end_elem.pop_next();
                // Unlink the end element from the element before it.
                debug_assert!(end_next.is_none());
                end_elem.pop_prev()
            })
            .unwrap();
        let new_start_end = if let Some(prev) = end_prev {
            // The list was originally longer than a singleton.
            let unlink_end = |prev_elem: &mut Chain<Item>| {
                let prev_next = prev_elem.pop_next();
                debug_assert_eq!(
                    prev_next.map(usize::from),
                    Some(usize::from(end.clone()))
                );
            };
            arena
                .map_mut(prev.clone(), unlink_end)
                .unwrap();
            Some((start, prev))
        } else {
            // The list was originally a singleton.
            None
        };
        // Update the start and end indices, and the length
        self.length -= 1;
        self.start_end = new_start_end;
        Some(end)
    }

    fn iter_base<'a, CA: Arena<Chain<Item>>>(
        &self,
        arena: &'a CA,
    ) -> IterBase<'a, Item, CA> {
        IterBase { start_end: self.start_end(), arena, phantom: PhantomData }
    }

    pub fn checked_iter<'a, CA: Arena<Chain<Item>>>(
        &self,
        arena: &'a CA,
    ) -> IterChecked<'a, Item, CA> {
        IterChecked { inner: self.iter_base(arena) }
    }

    pub fn iter<'a, CA: Arena<Chain<Item>>>(
        &self,
        arena: &'a CA,
    ) -> Iter<'a, Item, CA> {
        Iter { inner: self.checked_iter(arena) }
    }

    pub fn iter_with_chain<'a, CA: Arena<Chain<Item>>>(
        &self,
        arena: &'a CA,
    ) -> IterWithChain<'a, Item, CA> {
        IterWithChain { inner: self.iter_base(arena) }
    }
}

/// An iterator for which the elements consist of the pairs
/// `(Index<Item>, Index<Chain<Item>>)`.
#[derive(Debug, Clone)]
struct IterBase<'a, Item: ArenaItem, ChainArena: Arena<Chain<Item>>> {
    start_end: StartEnd<Item>,
    arena: &'a ChainArena,
    phantom: PhantomData<Item>,
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> Iterator
    for IterBase<'a, T, ChainArena>
{
    type Item = ArenaResult<(Index<T>, Index<Chain<T>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = self.start_end.clone()?;
        let mut cloned_start = match self
            .arena
            .map(start.clone(), Clone::clone)
        {
            Ok(cloned) => cloned,
            Err(err) => return Some(Err(err)),
        };
        let new_start_end = cloned_start
            .pop_next()
            .zip(Some(end.clone()))
            .filter(|_| start != end);
        self.start_end = new_start_end;
        Some(Ok((cloned_start.get_index(), start)))
    }
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> DoubleEndedIterator
    for IterBase<'a, T, ChainArena>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (start, end) = self.start_end.clone()?;
        let mut cloned_end = match self
            .arena
            .map(end.clone(), Clone::clone)
        {
            Ok(cloned) => cloned,
            Err(err) => return Some(Err(err)),
        };
        let new_start_end = Some(start.clone())
            .zip(cloned_end.pop_prev())
            .filter(|_| start != end);
        self.start_end = new_start_end;
        Some(Ok((cloned_end.get_index(), end)))
    }
}

/// An iterator for which the elements consist of `Index<Chain<Item>>`.
#[derive(Debug, Clone)]
pub struct IterWithChain<'a, Item: ArenaItem, ChainArena: Arena<Chain<Item>>> {
    inner: IterBase<'a, Item, ChainArena>,
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> Iterator
    for IterWithChain<'a, T, ChainArena>
{
    type Item = ArenaResult<Index<Chain<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|inner| inner.map(|i| i.1))
    }
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> DoubleEndedIterator
    for IterWithChain<'a, T, ChainArena>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|inner| inner.map(|i| i.1))
    }
}

/// An iterator for which the elements consist of `Index<Item>`.
#[derive(Debug, Clone)]
pub struct Iter<'a, Item: ArenaItem, ChainArena: Arena<Chain<Item>>> {
    inner: IterChecked<'a, Item, ChainArena>,
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> Iterator
    for Iter<'a, T, ChainArena>
{
    type Item = Index<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(ArenaResult::unwrap)
    }
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> DoubleEndedIterator
    for Iter<'a, T, ChainArena>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(ArenaResult::unwrap)
    }
}

/// An iterator for which the elements consist of `ArenaResult<Index<Item>>`.
#[derive(Debug, Clone)]
pub struct IterChecked<'a, Item: ArenaItem, ChainArena: Arena<Chain<Item>>> {
    inner: IterBase<'a, Item, ChainArena>,
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> Iterator
    for IterChecked<'a, T, ChainArena>
{
    type Item = ArenaResult<Index<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|inner| Ok(inner?.0))
    }
}

impl<'a, T: ArenaItem, ChainArena: Arena<Chain<T>>> DoubleEndedIterator
    for IterChecked<'a, T, ChainArena>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|inner| Ok(inner?.0))
    }
}
