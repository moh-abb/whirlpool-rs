//! Used for indicating whether traversal of a linked data structure will occur
//! mutably or immutably.

use core::ops::Deref;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

/// Indicates that visiting the data structure will occur immutably.
pub struct VisitRef;
/// Indicates that visiting the data structure will occur mutably.
pub struct VisitMut;

mod private {
    pub trait Sealed {}
}

impl private::Sealed for VisitRef {}
impl private::Sealed for VisitMut {}

pub(super) trait VisitType: private::Sealed {
    type Ref<'a, Node: 'a>: Deref<Target = Node>;

    fn with_ref<'a, T, Node: ArenaItem>(
        ref_type: Self::Ref<'a, Node>,
        func: impl FnOnce(Self::Ref<'_, Node>) -> T,
    ) -> (T, Self::Ref<'a, Node>);

    fn arena_map<T, Node: ArenaItem>(
        arena: &impl Arena<Node>,
        index: Index<Node>,
        func: impl FnOnce(Self::Ref<'_, Node>) -> T,
    ) -> ArenaResult<T>;
}

impl VisitType for VisitRef {
    type Ref<'a, Node: 'a> = &'a Node;

    fn with_ref<'a, T, Node: ArenaItem>(
        ref_type: Self::Ref<'a, Node>,
        func: impl FnOnce(Self::Ref<'_, Node>) -> T,
    ) -> (T, Self::Ref<'a, Node>) {
        (func(ref_type), ref_type)
    }

    fn arena_map<T, Node: ArenaItem>(
        arena: &impl Arena<Node>,
        index: Index<Node>,
        func: impl FnOnce(Self::Ref<'_, Node>) -> T,
    ) -> ArenaResult<T> {
        arena.map(index, func)
    }
}

impl VisitType for VisitMut {
    type Ref<'a, Node: 'a> = &'a mut Node;

    fn with_ref<'a, T, Node: ArenaItem>(
        ref_type: Self::Ref<'a, Node>,
        func: impl FnOnce(Self::Ref<'_, Node>) -> T,
    ) -> (T, Self::Ref<'a, Node>) {
        (func(ref_type), ref_type)
    }

    fn arena_map<T, Node: ArenaItem>(
        arena: &impl Arena<Node>,
        index: Index<Node>,
        func: impl FnOnce(Self::Ref<'_, Node>) -> T,
    ) -> ArenaResult<T> {
        arena.map_mut(index, func)
    }
}
