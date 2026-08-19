use core::marker::PhantomData;
use core::mem::swap;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::linked::Linked;

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

/// A structure for a doubly-linked list of multiple arena-allocated elements.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Multiple<Item: ArenaItem> {
    length: u16,
    start_end: StartEnd<Item>,
}

impl<Child: ArenaItem> Multiple<Child> {
    /// Creates a new empty [Multiple].
    pub fn new() -> Self {
        Self { length: 0, start_end: StartEnd::Empty }
    }

    pub fn start(&self) -> Option<Index<Child>> {
        let StartEnd::Full { start, end: _ } = self.start_end.clone() else {
            return None;
        };
        Some(start)
    }

    pub fn end(&self) -> Option<Index<Child>> {
        let StartEnd::Full { start: _, end } = self.start_end.clone() else {
            return None;
        };
        Some(end)
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        let is_empty = self.length == 0;
        debug_assert_eq!(matches!(self.start_end, StartEnd::Empty), is_empty);
        is_empty
    }

    /// Removes the child element from the arena at the given position, also
    /// unlinking it from its siblings, and returning the child.
    ///
    /// The child should currently have a set parent, and if the parent has
    /// more than one child, the element should have at least one sibling.
    pub fn remove<'a, Parent, ChildArena, ParentArena>(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        child_index: Index<Child>,
    ) -> ArenaResult<Child>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        // Take the child from the arena.
        let mut child = child_arena.take(child_index.clone())?;
        // Unlink the child from its siblings and parent.
        let parent_index = child
            .get_mut_parent()
            .take()
            .ok_or(ArenaError::ExpectedParent)?;
        let siblings = child.get_mut_sibling_chain();
        let opt_prev = siblings.prev.take();
        let opt_next = siblings.next.take();
        // Unlink the siblings from the child
        if let Some(prev) = opt_prev.clone() {
            child_arena.map_mut(prev, |elem| {
                let opt_next = elem.get_mut_sibling_chain().next.take();
                debug_assert_eq!(opt_next, Some(child_index.clone()));
            })?;
        }
        if let Some(next) = opt_next.clone() {
            child_arena.map_mut(next, |elem| {
                let opt_prev = elem.get_mut_sibling_chain().prev.take();
                debug_assert_eq!(opt_prev, Some(child_index.clone()));
            })?;
        }
        // Update the start and end of the `Multiple`.
        parent_arena.map_mut(parent_index, |parent| {
            let multiple = <Child as Linked<_, _, _>>::get_mut_children(parent)
                .ok_or(ArenaError::ExpectedChildren)?;
            match &mut multiple.start_end {
                StartEnd::Empty => {
                    debug_assert!(false);
                }
                StartEnd::Full { start, end } if start == end => {
                    // The list was originally a singleton but will now become
                    // empty.
                    debug_assert_eq!(&*start, &child_index);
                    multiple.start_end = StartEnd::Empty;
                }
                StartEnd::Full { start, end } => {
                    if *start == child_index {
                        *start = opt_next.ok_or(ArenaError::ExpectedSibling)?;
                    }
                    if *end == child_index {
                        *end = opt_prev.ok_or(ArenaError::ExpectedSibling)?;
                    }
                }
            }
            multiple.length -= 1;
            Ok(())
        })??;

        Ok(child)
    }

    /// Verifies the properties of the [Multiple] before insertion.
    /// See [insert] for more details.
    #[inline]
    fn verify_multiple_on_insert<'a, Parent, ChildArena, ParentArena>(
        child_arena: &ChildArena,
        parent_arena: &ParentArena,
        parent_index: &Index<Parent>,
        opt_prev: &Option<Index<Child>>,
        opt_next: &Option<Index<Child>>,
    ) where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        // Verify both prev and next have the right parent.
        let verify_parent = |child: &Index<_>| {
            debug_assert_eq!(
                child_arena
                    .map(child.clone(), |child| { child.get_parent().clone() }),
                Ok(Some(parent_index.clone()))
            );
        };
        if let Some(prev) = opt_prev {
            verify_parent(prev);
        }
        if let Some(next) = opt_next {
            verify_parent(next);
        }
        // Check the different cases of the prev and next.
        let verify_len_eq = |expected: _| {
            debug_assert_eq!(
                parent_arena.map(parent_index.clone(), |parent| {
                    Child::get_children(parent).map(Self::length)
                }),
                Ok(Some(expected))
            );
        };
        let verify_len_ne = |expected: _| {
            debug_assert_ne!(
                parent_arena.map(parent_index.clone(), |parent| {
                    Child::get_children(parent).map(Self::length)
                }),
                Ok(Some(expected))
            );
        };
        let verify_start_eq = |expected: Option<_>| {
            debug_assert_eq!(
                parent_arena.map(parent_index.clone(), |parent| {
                    Child::get_children(parent).map(Self::start)
                }),
                Ok(Some(expected))
            );
        };
        let verify_end_eq = |expected: Option<_>| {
            debug_assert_eq!(
                parent_arena.map(parent_index.clone(), |parent| {
                    Child::get_children(parent).map(Self::end)
                }),
                Ok(Some(expected))
            );
        };
        match (opt_prev.clone(), opt_next.clone()) {
            (None, None) => {
                // Multiple should be empty.
                verify_start_eq(None);
                verify_end_eq(None);
                verify_len_eq(0);
            }
            (Some(prev), None) => {
                // Should be appending to end.
                verify_end_eq(Some(prev));
            }
            (None, Some(next)) => {
                // Should be prepending to start.
                verify_start_eq(Some(next));
            }
            (Some(prev), Some(next)) => {
                // prev and next should be distinct.
                debug_assert_ne!(prev, next);
                // If we have two distinct elements, the list shouldn't be
                // empty or have length 1.
                verify_len_ne(0);
                verify_len_ne(1);
            }
        }

        let verify_elem_prev = |child: &Index<_>, expected_prev: Option<_>| {
            debug_assert_eq!(
                child_arena.map(child.clone(), |child| {
                    child.get_sibling_chain().prev()
                }),
                Ok(expected_prev)
            );
        };
        let verify_elem_next = |child: &Index<_>, expected_next: Option<_>| {
            debug_assert_eq!(
                child_arena.map(child.clone(), |child| {
                    child.get_sibling_chain().next()
                }),
                Ok(expected_next)
            );
        };
        match (opt_prev.clone(), opt_next.clone()) {
            (Some(prev), Some(next)) => {
                // Verify that prev->next == next and next->prev == prev.
                verify_elem_prev(&next, Some(prev.clone()));
                verify_elem_next(&prev, Some(next.clone()));
            }
            (None, Some(next)) => {
                // The last element should not have a prev sibling.
                verify_elem_prev(&next, None);
            }
            (Some(prev), None) => {
                // The first element should not have a next sibling.
                verify_elem_next(&prev, None);
            }
            (None, None) => (),
        }
    }

    /// Allocates and inserts the element between `prev` and `next`, returning
    /// the index pointing to `child`.
    ///
    /// Preconditions:
    /// - `parent`, `prev` and `next` must all come from the same arena.
    /// - `parent`, `prev` and `next` are all distinct.
    /// - If both `prev` and `next` are Some, then both `prev` and `next` are
    /// children of `parent`.
    /// - `child` has no parent or siblings attached.
    ///
    /// Postcondition:
    /// - If `child` is successfully allocated into the arena, and the
    /// indices provided by `prev` and `next` are valid, then `child` will have
    /// parent `parent` and siblings `prev` and `next`.
    ///
    /// Cases to consider:
    /// - If both `prev` and `next` are None, then the `Multiple` must be empty.
    /// - If `prev` is None but `next` is Some, then we must be inserting onto
    /// the start of the [Multiple].
    /// - If `prev` is Some and `next` is None, then we must be inserting onto
    /// the end of the [Multiple].
    /// - If both `prev` and `next` are Some, then `Multiple` must have size
    /// greater than 1.
    fn insert<'a, Parent, ChildArena, ParentArena>(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
        opt_prev: Option<Index<Child>>,
        opt_next: Option<Index<Child>>,
        mut child_cow: Cow<Child>,
    ) -> ArenaResult<Index<Child>>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        // Check that the child is currently unlinked.
        child_cow.map(child_arena, |child| {
            debug_assert_eq!(child.get_parent(), &None);
            debug_assert_eq!(child.get_sibling_chain().prev(), None);
            debug_assert_eq!(child.get_sibling_chain().next(), None);
        })?;
        // Check the different cases of the prev and next.
        Self::verify_multiple_on_insert(
            child_arena,
            parent_arena,
            &parent_index,
            &opt_prev,
            &opt_next,
        );

        // Prepare the child to be added into the list by setting its elements.
        // This way, if allocation succeeds, we don't have to modify the child
        // element again.
        child_cow.map_mut(child_arena, |child| {
            *child.get_mut_parent() = Some(parent_index.clone());
            let chain = child.get_mut_sibling_chain();
            chain.prev = opt_prev.clone();
            chain.next = opt_next.clone();
        })?;

        // Try to allocate the child element.
        let child_index = match child_cow {
            Cow::Indexed(index) => index,
            Cow::Owned(child) => child_arena.push(child)?,
        };

        // Update the prev and next elements to point to the child.
        if let Some(prev) = opt_prev.clone() {
            child_arena.map_mut(prev, |elem| {
                elem.get_mut_sibling_chain().next = Some(child_index.clone());
            })?;
        }
        if let Some(next) = opt_next.clone() {
            child_arena.map_mut(next, |elem| {
                elem.get_mut_sibling_chain().prev = Some(child_index.clone());
            })?;
        }

        let update_parent_ends = |parent: &mut Parent| {
            let multiple =
                &mut <Child as Linked<_, _, _>>::get_mut_children(parent)
                    .ok_or(ArenaError::ExpectedChildren)?;
            match &mut multiple.start_end {
                StartEnd::Empty => {
                    // The list will now become a singleton with both ends
                    // pointing to the element.
                    multiple.start_end = StartEnd::Full {
                        start: child_index.clone(),
                        end: child_index.clone(),
                    };
                }
                StartEnd::Full { start, end } => {
                    match (opt_prev.clone(), opt_next.clone()) {
                        (None, None) => debug_assert!(false),
                        (None, Some(_)) => *start = child_index.clone(),
                        (Some(_), None) => *end = child_index.clone(),
                        (Some(_), Some(_)) => (),
                    }
                }
            }
            multiple.length += 1;
            Ok(())
        };

        // Update the ends of the parent.
        parent_arena.map_mut(parent_index.clone(), update_parent_ends)??;
        Ok(child_index)
    }

    fn get_end_in_arena<Parent, ChildArena, ParentArena>(
        parent_arena: &ParentArena,
        parent_index: Index<Parent>,
        get_end: impl FnOnce(&Self) -> Option<Index<Child>>,
    ) -> ArenaResult<Option<Index<Child>>>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        parent_arena.map(parent_index.clone(), |parent| {
            let multiple = <Child as Linked<_, _, _>>::get_children(parent)
                .ok_or(ArenaError::ExpectedChildren)?;
            Ok(get_end(multiple))
        })?
    }

    /// Prepends a child to the front of its parent's list.
    /// `child` must initially have no parent, and have no previous or
    /// next element.
    /// The child node will be allocated into the arena.
    /// After prepending, the parent will see that it has a new start child
    /// node, and the child will see that its parent is the one provided.
    /// Returns the allocated child index.
    pub fn push_front<Parent, ChildArena, ParentArena>(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
        child_cow: Cow<Child>,
    ) -> ArenaResult<Index<Child>>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        let opt_start = Self::get_end_in_arena(
            parent_arena,
            parent_index.clone(),
            Self::start,
        )?;
        Self::insert(
            child_arena,
            parent_arena,
            parent_index,
            None,
            opt_start,
            child_cow,
        )
    }

    /// Appends a child to the end of its parent's list.
    /// `child` must initially have no parent, and have no previous or
    /// next element.
    /// The child node will be allocated into the arena.
    /// After appending, the parent will see that it has a new end child
    /// node, and the child will see that its parent is the one provided.
    /// Returns the allocated child index.
    pub fn push_back<Parent, ChildArena, ParentArena>(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
        child_cow: Cow<Child>,
    ) -> ArenaResult<Index<Child>>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        let opt_end = Self::get_end_in_arena(
            parent_arena,
            parent_index.clone(),
            Self::end,
        )?;
        Self::insert(
            child_arena,
            parent_arena,
            parent_index,
            opt_end,
            None,
            child_cow,
        )
    }

    /// Pops a child from the front of its parent's list.
    /// After removal, `child` will see its parent be `None`,
    /// and the parent will no longer have its start element as `child`.
    /// Returns the optional removed child.
    pub fn pop_front<Parent, ChildArena, ParentArena>(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
    ) -> ArenaResult<Option<Child>>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        let opt_front =
            Self::get_end_in_arena(parent_arena, parent_index, Self::start)?;
        let Some(front) = opt_front else {
            return Ok(None);
        };
        Self::remove(child_arena, parent_arena, front).map(Some)
    }

    /// Pops a child from the end of its parent's list.
    /// After removal, `child` will see its parent be the sentinel invalid
    /// index, and the parent will no longer have its end element as `child`.
    /// Returns the optional removed child.
    pub fn pop_back<Parent, ChildArena, ParentArena>(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
    ) -> ArenaResult<Option<Child>>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        let opt_end =
            Self::get_end_in_arena(parent_arena, parent_index, Self::end)?;
        let Some(end) = opt_end else {
            return Ok(None);
        };
        Self::remove(child_arena, parent_arena, end).map(Some)
    }

    fn iter_base<'a, Parent, ChildArena, ParentArena>(
        &self,
        child_arena: &'a ChildArena,
    ) -> IterBase<'a, Child, Parent, ChildArena, ParentArena>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        IterBase {
            start_end: self.start_end.clone(),
            child_arena,
            phantom: PhantomData,
        }
    }

    pub fn checked_iter<'a, Parent, ChildArena, ParentArena>(
        &self,
        child_arena: &'a ChildArena,
    ) -> IterChecked<'a, Child, Parent, ChildArena, ParentArena>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        IterChecked { inner: self.iter_base(child_arena) }
    }

    pub fn unwrapped_iter<'a, Parent, ChildArena, ParentArena>(
        &self,
        child_arena: &'a ChildArena,
    ) -> IterUnwrapped<'a, Child, Parent, ChildArena, ParentArena>
    where
        Parent: ArenaItem,
        ChildArena: Arena<Child>,
        ParentArena: Arena<Parent>,
        Child: Linked<Parent, ChildArena, ParentArena>,
    {
        IterUnwrapped { inner: self.iter_base(child_arena) }
    }
}

/// An iterator for which the elements consist of the pairs
/// `(Index<Item>, Index<Chain<Item>>)`.
#[derive(Debug, Clone)]
struct IterBase<'a, Child, Parent, ChildArena, ParentArena> {
    start_end: StartEnd<Child>,
    child_arena: &'a ChildArena,
    phantom: PhantomData<(Parent, ParentArena)>,
}

impl<'a, Child, Parent, ChildArena, ParentArena>
    IterBase<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    fn next_or_next_back(
        &mut self,
        is_next_back: bool,
    ) -> Option<ArenaResult<Index<Child>>> {
        let cloned_start_end = self.start_end.clone();
        let StartEnd::Full { start, end } = cloned_start_end else {
            return None;
        };
        let (mut changing_end, mut constant_end) = (start, end);
        if is_next_back {
            swap(&mut changing_end, &mut constant_end);
        }

        let result = self
            .child_arena
            .map(changing_end.clone(), |child| {
                child.get_sibling_chain().clone()
            })
            .map(|chain| {
                let old_end = changing_end.clone();
                self.start_end = if is_next_back {
                    if let Some(prev) = chain.prev {
                        StartEnd::Full { start: constant_end, end: prev }
                    } else {
                        StartEnd::Empty
                    }
                } else {
                    if let Some(next) = chain.next {
                        StartEnd::Full { start: next, end: constant_end }
                    } else {
                        StartEnd::Empty
                    }
                };
                old_end
            });
        Some(result)
    }
}

impl<'a, Child, Parent, ChildArena, ParentArena> Iterator
    for IterBase<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    type Item = ArenaResult<Index<Child>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_or_next_back(false)
    }
}

impl<'a, Child, Parent, ChildArena, ParentArena> DoubleEndedIterator
    for IterBase<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_or_next_back(true)
    }
}

/// An iterator for which the elements consist of `Index<Item>`.
#[derive(Debug, Clone)]
pub struct IterUnwrapped<'a, Child, Parent, ChildArena, ParentArena> {
    inner: IterBase<'a, Child, Parent, ChildArena, ParentArena>,
}

impl<'a, Child, Parent, ChildArena, ParentArena> Iterator
    for IterUnwrapped<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    type Item = Index<Child>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(ArenaResult::unwrap)
    }
}

impl<'a, Child, Parent, ChildArena, ParentArena> DoubleEndedIterator
    for IterUnwrapped<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(ArenaResult::unwrap)
    }
}

/// An iterator for which the elements consist of `ArenaResult<Index<Item>>`.
#[derive(Debug, Clone)]
pub struct IterChecked<'a, Child, Parent, ChildArena, ParentArena> {
    inner: IterBase<'a, Child, Parent, ChildArena, ParentArena>,
}

impl<'a, Child, Parent, ChildArena, ParentArena> Iterator
    for IterChecked<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    type Item = ArenaResult<Index<Child>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, Child, Parent, ChildArena, ParentArena> DoubleEndedIterator
    for IterChecked<'a, Child, Parent, ChildArena, ParentArena>
where
    Child: ArenaItem,
    Parent: ArenaItem,
    ChildArena: Arena<Child>,
    ParentArena: Arena<Parent>,
    Child: Linked<Parent, ChildArena, ParentArena>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
