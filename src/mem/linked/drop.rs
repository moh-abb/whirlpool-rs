use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::linked::Linked;
use crate::mem::linked::traversal_state::TraversalState;
use crate::mem::linked::visit_linked;
use crate::mem::linked::visit_type::VisitMut;

struct DropTraversalState<'a, Arenas> {
    arenas: &'a Arenas,
    result: ArenaResult<()>,
}

impl<Node, Arenas> TraversalState<Node> for DropTraversalState<'_, Arenas>
where
    Node: Linked<Node, Arenas>,
{
    type Visit = VisitMut;
    type Output = ();

    fn enter_node(&mut self, _: &mut Node) -> Self::Output {}

    fn exit_node(&mut self, cur_node: &mut Node) -> Self::Output {}

    fn post_enter(&mut self, _: Index<Node>) {}

    fn post_exit(&mut self, index: Index<Node>) {
        // Remove the item from the arena.
        let take_result = Node::child_arena(self.arenas)
            .take(index)
            .map(|_| ());
        self.result = self.result.or(take_result);
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
pub fn drop_linked<Node, Arenas>(
    start_index: Index<Node>,
    arenas: &Arenas,
) -> ArenaResult<()>
where
    Node: Linked<Node, Arenas>,
{
    let start_state = DropTraversalState { arenas, result: Ok(()) };
    let final_state = visit_linked(start_index, start_state, arenas)?;
    final_state.result
}
