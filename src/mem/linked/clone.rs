use core::marker::PhantomData;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::SharedArena;
use crate::mem::SharedArenaRef;
use crate::mem::linked::Linked;
use crate::mem::linked::drop::drop_linked;
use crate::mem::linked::traversal_state::TraversalState;
use crate::mem::linked::visit_type::VisitRef;
use crate::mem::linked::visitor::visit_linked;

/// Used to implement a FSM to represent the current state of cloning as we
/// traverse the source structure.
/// In the normal case, the [CloneHandler] alternates between `Ready` and
/// `Entered`:
/// - Before entering the first source node, the state is `Ready` with an
/// empty parent.
/// - When entering a source node, the node is copied to `src_node` and the
/// destination parent is recorded.
/// - On `post_enter`, we attempt to allocate the new node and link it to the
/// parent. This needs to happen in `post_enter` otherwise we may attempt to
/// modify the [Arena] while it is already borrowed.
///     - If allocation is successful, then on entering a child node, the
/// index that was just allocated will become the new `dest_parent`.
///     - If allocation failed, then the error is recorded in `Failed`.
/// - On `post_exit`, we update the parent node to be the parent of the current
/// parent (assuming it still exists, which should be the case).
struct CloneHandler<'a, Node, SourceArena, DestArena, CloneF> {
    dest_arena: &'a mut DestArena,
    copied_node: Option<Node>,
    dest_parent: Option<Index<Node>>,
    dest_root: Option<Index<Node>>,
    clone_handler: CloneF,
    phantom: PhantomData<SourceArena>,
}

impl<'a, Node, SourceArena, DestArena, CloneF> TraversalState<Node>
    for CloneHandler<'a, Node, SourceArena, DestArena, CloneF>
where
    SourceArena: Arena<Node>,
    DestArena: Arena<Node>,
    for<'r, 'b> Node: Linked<
            Node,
            SharedArenaRef<'r, &'b mut DestArena>,
            SharedArenaRef<'r, &'b mut DestArena>,
        >,
    CloneF: FnMut(&Node) -> Node,
{
    type Visit = VisitRef;
    type Output = ();
    type Error = ArenaError;

    fn enter_node(
        &mut self,
        cur_node: &Node,
    ) -> Result<Self::Output, Self::Error> {
        debug_assert!(self.copied_node.is_none());

        let mut copied_node = (self.clone_handler)(cur_node);
        // `Multiple` will allocate and push back the element.
        // To prepare, clear the copied node's parent and sibling
        // indices.
        copied_node.get_mut_parent().take();
        *copied_node.get_mut_sibling_chain() = Chain::new();
        if let Some(multiple) = Node::get_mut_children(&mut copied_node) {
            *multiple = Multiple::new();
        }

        self.copied_node = Some(copied_node);
        Ok(())
    }

    fn exit_node(&mut self, _: &Node) -> Result<Self::Output, Self::Error> {
        // Indicate true to leave successfully; we never repeat iteration.
        Ok(())
    }

    fn post_enter(
        &mut self,
        _: Index<Node>,
        _: &impl Arena<Node>,
    ) -> Result<(), Self::Error> {
        // Take the node we got when we were still inside the arena slot.
        let copied_node = self
            .copied_node
            .take()
            .ok_or(ArenaError::InvariantBroken)?;

        // Allocate the child node, this will be the "next parent" when
        // traversing downwards
        let next_parent = if let Some(parent) = self.dest_parent.take() {
            let shared_arena = SharedArena::new(&mut *self.dest_arena);
            let mut shared_ref_1 = shared_arena.make_ref();
            let mut shared_ref_2 = shared_arena.make_ref();
            let new_node_index = Multiple::push_back(
                &mut shared_ref_1,
                &mut shared_ref_2,
                parent,
                Cow::Owned(copied_node),
            )?;
            // We might enter the index's children: update the parent
            new_node_index
        } else {
            // Just allocate the root node on its own because there cannot
            // be any sibling nodes that need to be modified.
            let new_node_index = self.dest_arena.push(copied_node)?;

            // Don't forget to store the root node as well.
            self.dest_root = Some(new_node_index.clone());

            new_node_index
        };

        self.dest_parent = Some(next_parent);
        Ok(())
    }

    fn post_exit(
        &mut self,
        _: Index<Node>,
        _: &impl Arena<Node>,
    ) -> Result<(), Self::Error> {
        // We are moving back up, so update the node to be the parent
        let Some(parent) = self.dest_parent.take() else {
            return Err(ArenaError::ExpectedParent);
        };

        self.dest_parent = self
            .dest_arena
            .map(parent, |node| node.get_parent().clone())?;

        Ok(())
    }
}

/// Clones a linked data structure from arena X into arena Y, given a function
/// that clones each element.
/// On success, the root node will not have a parent even if the original
/// element at `start_index` did.
///
/// The source structure will be traversed, and the clone function will be
/// called on each element. Then the cloned element has its source parent and
/// siblings removed and replaced with the destination parent and siblings
/// in the new arena.
///
/// Because allocation can fail once the arena reaches capacity, the result
/// may need to be dropped (in which case `drop_handler` will be called on
/// each element and the partial result will be removed).
pub fn clone_linked<Node, SourceArena, DestArena>(
    start_index: Index<Node>,
    source_arena: &SourceArena,
    dest_arena: &mut DestArena,
    clone_handler: impl FnMut(&Node) -> Node,
) -> ArenaResult<Index<Node>>
where
    SourceArena: Arena<Node>,
    DestArena: Arena<Node>,
    Node: Linked<Node, SourceArena, SourceArena>,
    Node: Linked<Node, DestArena, DestArena>,
    for<'r, 'b> Node: Linked<
            Node,
            SharedArenaRef<'r, &'b mut DestArena>,
            SharedArenaRef<'r, &'b mut DestArena>,
        >,
{
    let mut state = CloneHandler {
        dest_arena,
        copied_node: None,
        dest_parent: None,
        dest_root: None,
        clone_handler,
        phantom: PhantomData::<SourceArena>,
    };
    let visit_result = visit_linked(start_index, &mut state, source_arena);

    if let Err(err) = visit_result {
        if let Some(root) = state.dest_root {
            drop_linked(root, dest_arena)?;
        }
        return Err(err);
    }

    // Entering the node for the first time should change the clone state to
    // have a destination root
    state
        .dest_root
        .ok_or(ArenaError::InvariantBroken)
}
