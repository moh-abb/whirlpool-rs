use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;

pub mod traversal_state;
pub mod visit_type;
pub mod visitor;

pub use traversal_state::TraversalState;
pub use traversal_state::UncondTravState;
pub use visit_type::VisitMut;
pub use visit_type::VisitRef;
pub use visitor::controlled_visit_linked;
pub use visitor::visit_linked;
pub use visitor::visit_linked_mut;

/// Represents a structure which is linked to other elements such that:
/// - It can have a parent which owns its children via a [Multiple]
/// - It can have siblings which are referenced via a [Chain]
///
/// Note that the parent doesn't necessarily have to be of the same type:
/// There could be an AST node "Program" which has multiple child nodes
/// "Statement", such that the "Statement"'s parent is the "Program".
/// Here, we would implement [Linked] with `Child` as `Statement` and
/// `Parent` as `Program`.
pub trait Linked<Parent, Arenas>
where
    Self: ArenaItem + Sized,
    Parent: ArenaItem,
{
    fn parent_arena(arenas: &Arenas) -> &impl Arena<Parent>;
    fn child_arena(arenas: &Arenas) -> &impl Arena<Self>;

    fn get_parent(&self) -> &Option<Index<Parent>>;
    fn get_mut_parent(&mut self) -> &mut Option<Index<Parent>>;

    fn get_sibling_chain(&self) -> &Chain<Self>;
    fn get_mut_sibling_chain(&mut self) -> &mut Chain<Self>;

    fn get_children(parent: &Parent) -> Option<&Multiple<Self>>;
    fn get_mut_children(parent: &mut Parent) -> Option<&mut Multiple<Self>>;
}
