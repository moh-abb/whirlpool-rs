use core::marker::PhantomData;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::linked::Linked;
use crate::mem::linked::VisitMut;
use crate::mem::linked::traversal_state::TraversalState;
use crate::mem::linked::visit_linked_mut;

struct DropTraversalState<Arenas>(PhantomData<Arenas>);

impl<Node, NodeArena> TraversalState<Node> for DropTraversalState<NodeArena>
where
    NodeArena: Arena<Node>,
    Node: Linked<Node, NodeArena, NodeArena>,
{
    type Visit = VisitMut;
    type Output = ();
    type Error = ArenaError;

    fn enter_node(
        &mut self,
        _: &mut Node,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn exit_node(&mut self, _: &mut Node) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn post_exit(
        &mut self,
        index: Index<Node>,
        arena: &mut impl Arena<Node>,
    ) -> Result<(), Self::Error> {
        arena.take(index).map(|_| ())
    }
}

/// Drops a linked data structure by visiting it.
/// Each node will be processed in the same order dictated by [visit_linked],
/// and when the node is exited, the drop handler is ran (to allow for
/// additional logic before removal from the arena), and finally the node is
/// removed from the arena via [Arena::take]. No changes occur when the node
/// is entered.
///
/// Note that after dropping, the reference to `start_index` will become
/// invalid, and any objects holding references to `start_index` will need to
/// remove this reference.
pub fn drop_linked<Node, NodeArena>(
    start_index: Index<Node>,
    arenas: &mut NodeArena,
) -> ArenaResult<()>
where
    NodeArena: Arena<Node>,
    Node: Linked<Node, NodeArena, NodeArena>,
{
    let mut start_state = DropTraversalState(PhantomData);
    visit_linked_mut(start_index, &mut start_state, arenas)
}
