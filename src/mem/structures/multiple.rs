use core::marker::PhantomData;
use core::mem::swap;

use derive_where::derive_where;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::linked::Linked;

#[derive(derive_more::Debug)]
#[derive_where(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StartEnd<Child> {
    #[derive_where(default)]
    Empty,
    Full {
        start: Index<Child>,
        end: Index<Child>,
    },
}

/// A structure for a doubly-linked list of multiple arena-allocated elements.
#[derive(derive_more::Debug)]
#[derive_where(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Multiple<Item> {
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

    /// Points the end of the list to `child_index`.
    /// Preconditions:
    /// - `child_index.prev == self.end`
    /// - `child_index.next == None`
    /// - `child_index.parent == (the index of the entry that contains self)`
    pub(super) fn push_back(&mut self, child_index: Index<Child>) {
        match &mut self.start_end {
            StartEnd::Empty => {
                // The list will now become a singleton with both ends
                // pointing to the element.
                self.start_end = StartEnd::Full {
                    start: child_index.clone(),
                    end: child_index.clone(),
                };
            }
            StartEnd::Full { end, .. } => {
                *end = child_index.clone();
            }
        }
        self.length = self.length.wrapping_add(1);
    }

    /// Points the end of the list to `opt_prev`.
    /// Preconditions:
    /// - `self.end.prev == opt_prev`
    /// - If `self.length >= 2`, `opt_prev == Some(_)`
    /// - If `self.length < 2`, `opt_prev == None`
    /// To meet these preconditions, we need to unlink the current `end` of
    /// `self` from its previous sibling and parent, and call with `end.prev`.
    pub(super) fn pop_back(&mut self, opt_prev: Option<Index<Child>>) {
        match &mut self.start_end {
            StartEnd::Empty => {
                debug_assert!(opt_prev.is_none());
                return;
            }
            StartEnd::Full { start, end } if start == end => {
                // The list was a singleton but will now become empty.
                debug_assert!(opt_prev.is_none());
                self.start_end = StartEnd::Empty;
            }
            StartEnd::Full { end, .. } => {
                debug_assert!(opt_prev.is_some());
                if let Some(prev) = opt_prev {
                    *end = prev;
                }
            }
        }
        self.length = self.length.wrapping_sub(1);
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
        IterChecked {
            start_end: self.start_end.clone(),
            child_arena,
            phantom: PhantomData,
        }
    }
}

/// An iterator for which the elements consist of `ArenaResult<Index<Item>>`.
#[derive(Debug, Clone)]
pub struct IterChecked<'a, Child, Parent, ChildArena, ParentArena> {
    start_end: StartEnd<Child>,
    child_arena: &'a ChildArena,
    phantom: PhantomData<(Parent, ParentArena)>,
}

impl<'a, Child, Parent, ChildArena, ParentArena>
    IterChecked<'a, Child, Parent, ChildArena, ParentArena>
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
        self.next_or_next_back(false)
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
        self.next_or_next_back(true)
    }
}
