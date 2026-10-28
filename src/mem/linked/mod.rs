use derive_where::derive_where;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;

pub mod checker;
pub mod clone;
pub mod cmp;
pub mod drop;
pub mod traversal_state;
pub mod visit_type;
pub mod visitor;

#[derive(derive_more::Debug)]
#[debug(bound())]
#[derive_where(PartialEq, Eq, PartialOrd, Ord)]
pub enum ChildRef<'a, T> {
    Multiple(&'a Multiple<T>),
    Single(&'a Option<Index<T>>),
}

#[derive(derive_more::Debug)]
#[debug(bound())]
#[derive_where(PartialEq, Eq, PartialOrd, Ord)]
pub enum ChildMut<'a, T> {
    Multiple(&'a mut Multiple<T>),
    Single(&'a mut Option<Index<T>>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinkedNode<Item, Node, Parent> {
    pub parent: Option<Index<Parent>>,
    pub sibling_chain: Chain<Node>,
    pub item: Item,
}

impl<Item, Node, Parent> LinkedNode<Item, Node, Parent> {
    pub const fn new(item: Item) -> Self {
        Self { parent: None, sibling_chain: Chain::new(), item }
    }
}

/// Represents a structure which is linked to other elements such that:
/// - It can have a parent which owns its children via a [Multiple]
/// - It can have siblings which are referenced via a [Chain]
///
/// Note that the parent doesn't necessarily have to be of the same type:
/// There could be an AST node "Program" which has multiple child nodes
/// "Statement", such that the "Statement"'s parent is the "Program".
/// Here, we would implement [Linked] with `Child` as `Statement` and
/// `Parent` as `Program`.
pub trait Linked<Parent, ChildArena, ParentArena>
where
    Self: ArenaItem + Sized,
    Parent: ArenaItem + Sized,
    ChildArena: Arena<Self>,
    ParentArena: Arena<Parent>,
{
    fn get_parent(&self) -> &Option<Index<Parent>>;
    fn get_mut_parent(&mut self) -> &mut Option<Index<Parent>>;

    fn get_sibling_chain(&self) -> &Chain<Self>;
    fn get_mut_sibling_chain(&mut self) -> &mut Chain<Self>;

    fn get_children(parent: &Parent) -> Option<ChildRef<'_, Self>>;
    fn get_mut_children(parent: &mut Parent) -> Option<ChildMut<'_, Self>>;
}
