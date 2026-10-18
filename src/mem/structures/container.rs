use core::mem::replace;
use core::ops::ControlFlow;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::linked::ChildMut;
use crate::mem::linked::ChildRef;
use crate::mem::linked::Linked;

pub trait LinkedContainer<Parent, ChildArena, ParentArena>
where
    Parent: ArenaItem,
    ChildArena: Arena<Self>,
    ParentArena: Arena<Parent>,
    Self: Linked<Parent, ChildArena, ParentArena>,
{
    fn push_child(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
        mut child_cow: Cow<Self>,
    ) -> ArenaResult<Index<Self>> {
        // Check that the child is currently unlinked.
        child_cow.map(child_arena, |child| {
            debug_assert_eq!(child.get_parent(), &None);
            debug_assert_eq!(child.get_sibling_chain().prev(), None);
            debug_assert_eq!(child.get_sibling_chain().next(), None);
        })?;
        // Prepare the child to be added into the list / child by setting:
        // - Its parent (for both one or multiple children)
        // - Its previous sibling (for multiple children)
        // We can only push and pop from the (right) end, so the next sibling
        // on insertion is always `None`.
        let try_opt_prev =
            parent_arena.map(parent_index.clone(), |parent| {
                let children = Self::get_children(parent)
                    .ok_or(ArenaError::ExpectedChildren)?;
                // Don't allow adding to a single child after one has already
                // been set.
                Ok(match children {
                    ChildRef::Multiple(multiple) => {
                        ControlFlow::Continue(multiple.end())
                    }
                    ChildRef::Single(None) => ControlFlow::Continue(None),
                    ChildRef::Single(Some(_)) => ControlFlow::Break(()),
                })
            })??;
        let ControlFlow::Continue(opt_prev) = try_opt_prev else {
            return Err(ArenaError::ExpectedEmptyChild);
        };
        child_cow.map_mut(child_arena, |child| {
            *child.get_mut_parent() = Some(parent_index.clone());
            child.get_mut_sibling_chain().prev = opt_prev.clone();
        })?;
        // Try to allocate the child element.
        let child_index = match child_cow {
            Cow::Indexed(index) => index,
            Cow::Owned(child) => child_arena.push(child)?,
        };
        // If we have a previous element, make it point to our newly allocated
        // index.
        if let Some(prev) = opt_prev.clone() {
            child_arena.map_mut(prev, |previous| {
                previous.get_mut_sibling_chain().next =
                    Some(child_index.clone());
            })?;
        }
        // Perform another access on the parent to update the value of its
        // child.
        parent_arena.map_mut(parent_index.clone(), |parent| {
            let children = Self::get_mut_children(parent)
                .ok_or(ArenaError::ExpectedChildren)?;
            Ok(match children {
                ChildMut::Multiple(multiple) => {
                    multiple.push_back(child_index.clone())
                }
                ChildMut::Single(opt_index) => {
                    debug_assert!(opt_index.is_none());
                    *opt_index = Some(child_index.clone());
                }
            })
        })??;
        // Return the allocated child index.
        Ok(child_index)
    }

    fn pop_child(
        child_arena: &mut ChildArena,
        parent_arena: &mut ParentArena,
        parent_index: Index<Parent>,
        take_owned: bool,
    ) -> ArenaResult<Option<Cow<Self>>> {
        // Unlink the current end of the list, or the singular child.
        let opt_cur_end =
            parent_arena.map(parent_index.clone(), |parent| {
                let children = Self::get_children(parent)
                    .ok_or(ArenaError::ExpectedChildren)?;
                Ok(match children {
                    ChildRef::Multiple(multiple) => multiple.end(),
                    ChildRef::Single(opt_index) => opt_index.clone(),
                })
            })??;
        // If we have no last element, we can simply return `None`.
        let Some(cur_end) = opt_cur_end else {
            return Ok(None);
        };
        // Unlink the current end from its siblings and parent, and get the
        // previous sibling (if there is one).
        // We can only push and pop from the (right) end, so the next sibling
        // on removal is always `None`.
        let opt_prev = child_arena.map_mut(cur_end.clone(), |child| {
            debug_assert!(child.get_parent().is_some());
            *child.get_mut_parent() = None;
            let sibling_chain = child.get_mut_sibling_chain();
            debug_assert_eq!(sibling_chain.next(), None);
            // We now only need to unlink the previous element and return it.
            replace(&mut sibling_chain.prev, None)
        })?;
        // Update the parent to point to the optional previous element
        // (in a list) or to `None` (for the single child).
        parent_arena.map_mut(parent_index.clone(), |parent| {
            let children = Self::get_mut_children(parent)
                .ok_or(ArenaError::ExpectedChildren)?;
            Ok(match children {
                ChildMut::Multiple(multiple) => multiple.pop_back(opt_prev),
                ChildMut::Single(opt_index) => {
                    debug_assert!(opt_index.is_some());
                    *opt_index = None;
                }
            })
        })??;
        // Take the child from the arena if we have requested the owned version
        // and return it (otherwise, just the index).
        let child_cow = if take_owned {
            Cow::Owned(child_arena.take(cur_end)?)
        } else {
            Cow::Indexed(cur_end)
        };
        Ok(Some(child_cow))
    }
}

impl<Child, Parent, ChildArena, ParentArena>
    LinkedContainer<Parent, ChildArena, ParentArena> for Child
where
    Parent: ArenaItem,
    ChildArena: Arena<Self>,
    ParentArena: Arena<Parent>,
    Self: Linked<Parent, ChildArena, ParentArena>,
{
}
