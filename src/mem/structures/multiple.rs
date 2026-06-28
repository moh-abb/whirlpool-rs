use core::marker::PhantomData;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::structures::linked::Linked;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StartEnd<Child> {
    Empty,
    Full { start: Index<Child>, end: Index<Child> },
}

impl<Child> Clone for StartEnd<Child> {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Full { start, end } => {
                Self::Full { start: start.clone(), end: end.clone() }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Multiple<Item: ArenaItem> {
    length: u16,
    start_end: StartEnd<Item>,
}

impl<Child: ArenaItem> Multiple<Child> {
    pub fn new_nonempty<Parent, Arenas>(
        length: u16,
        start: Index<Child>,
        end: Index<Child>,
        arenas: &Arenas,
        make_parent: impl FnOnce(Multiple<Child>) -> Parent,
    ) -> ArenaResult<Index<Parent>>
    where
        Parent: ArenaItem,
        Child: Linked<Parent, Arenas>,
    {
        let parent_arena = <Child as Linked<_, _>>::parent_arena(arenas);
        let child_arena = <Child as Linked<_, _>>::child_arena(arenas);
        // Link the start and end to the parent.
        // First, copy the two ends.
        let mut copied_start = Linked::copy_child(start.clone(), arenas)?;
        let mut copied_end = Linked::copy_child(end.clone(), arenas)?;
        let copied_start_chain = copied_start.get_mut_sibling_chain();
        let copied_end_chain = copied_end.get_mut_sibling_chain();
        debug_assert!(copied_start_chain.prev.is_none());
        debug_assert!(copied_end_chain.next.is_none());
        // Create the item enclosing the `Multiple`.
        // We assume that if allocating this item fails, then no
        // irreversible changes have occurred.
        let multiple = Self {
            length,
            start_end: StartEnd::Full {
                start: start.clone(),
                end: end.clone(),
            },
        };
        let parent = make_parent(multiple);
        let parent_index = parent_arena.push(parent)?;
        // Then update the start and end to point to the parent
        let update_parent = |child: &mut Child| {
            let cur_parent_index = child.get_mut_parent();
            debug_assert!(*cur_parent_index == Index::new_invalid());
            *cur_parent_index = parent_index.clone();
        };
        child_arena.map_mut(start, update_parent)?;
        child_arena.map_mut(end, update_parent)?;
        // Return the allocated item index
        Ok(parent_index)
    }

    pub fn new_empty() -> Self {
        Self { length: 0, start_end: StartEnd::Empty }
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    fn start_end(&self) -> StartEnd<Child> {
        self.start_end.clone()
    }

    pub fn is_empty(&self) -> bool {
        let is_empty = self.length == 0;
        debug_assert_eq!(matches!(self.start_end, StartEnd::Empty), is_empty);
        is_empty
    }

    /// Appends a child to the front of its parent's list.
    /// `child` must hold a valid index to the parent, so that the parent
    /// will see that it has a new start child node.
    pub fn push_front<Parent, Arenas>(
        mut child: Child,
        arenas: &Arenas,
    ) -> ArenaResult<()>
    where
        Parent: ArenaItem,
        Child: Linked<Parent, Arenas>,
    {
        let parent_arena = <Child as Linked<_, _>>::parent_arena(arenas);
        let child_arena = <Child as Linked<_, _>>::child_arena(arenas);
        // The child should have a valid parent index.
        let parent_index = child.get_parent().clone();
        let mut taken_parent = parent_arena.take(parent_index.clone())?;
        // The child should not be linked already.
        let child_chain = child.get_sibling_chain();
        debug_assert!(child_chain.prev.is_none());
        debug_assert!(child_chain.next.is_none());
        // Try to allocate the child element
        let child_index = child_arena.push(child)?;
        // Get the parent's indices
        let parent_multiple =
            <Child as Linked<_, _>>::get_mut_children(&mut taken_parent);
        match &mut parent_multiple.start_end {
            StartEnd::Empty => {
                // The list will now become a singleton with both ends pointing
                // to the element.
                parent_multiple.start_end = StartEnd::Full {
                    start: child_index.clone(),
                    end: child_index.clone(),
                }
            }
            StartEnd::Full { start, end: _ } => {
                // Update `child` to point forwards to `start`
                let mut copied_child = child_arena.take(child_index.clone())?;
                let child_chain = copied_child.get_mut_sibling_chain();
                debug_assert!(child_chain.next.is_none());
                child_chain.next = Some(start.clone());
                // Update the element at `start` to point back to `child`
                let mut copied_start = child_arena.take(start.clone())?;
                let start_chain = copied_start.get_mut_sibling_chain();
                debug_assert!(start_chain.prev.is_none());
                start_chain.prev = Some(child_index.clone());
                // Copy the elements back to the arenas
                child_arena.insert()
                Linked::update_child(
                    child_index.clone(),
                    copied_child,
                    arenas,
                )?;
                Linked::update_child(start.clone(), copied_start, arenas)?;
                // Update `start` to now point to `child`
                *start = child_index;
            }
        }
        // Increment the length
        parent_multiple.length += 1;
        // Copy the parent back
        <Child as Linked<_, _>>::update_parent(
            parent_index,
            taken_parent,
            arenas,
        )?;
        Ok(())
    }

    pub fn push_back(
        &mut self,
        arena: &impl Arena<Chain<Child>>,
        elem_index: Index<Chain<Child>>,
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
            let link_end_to_new_end = |end_elem: &mut Chain<Child>| {
                end_elem.set_next(new_end.clone());
            };
            let link_new_end_to_end = |new_end_elem: &mut Chain<Child>| {
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
        arena: &impl Arena<Chain<Child>>,
    ) -> Option<Index<Chain<Child>>> {
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
            let unlink_end = |prev_elem: &mut Chain<Child>| {
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

    fn iter_base<'a, CA: Arena<Chain<Child>>>(
        &self,
        arena: &'a CA,
    ) -> IterBase<'a, Child, CA> {
        IterBase { start_end: self.start_end(), arena, phantom: PhantomData }
    }

    pub fn checked_iter<'a, CA: Arena<Chain<Child>>>(
        &self,
        arena: &'a CA,
    ) -> IterChecked<'a, Child, CA> {
        IterChecked { inner: self.iter_base(arena) }
    }

    pub fn iter<'a, CA: Arena<Chain<Child>>>(
        &self,
        arena: &'a CA,
    ) -> Iter<'a, Child, CA> {
        Iter { inner: self.checked_iter(arena) }
    }

    pub fn iter_with_chain<'a, CA: Arena<Chain<Child>>>(
        &self,
        arena: &'a CA,
    ) -> IterWithChain<'a, Child, CA> {
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
