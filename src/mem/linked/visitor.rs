use core::ops::ControlFlow;
use core::ops::ControlFlow::Break;
use core::ops::ControlFlow::Continue;
use core::ops::Deref;

use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::linked::Linked;
use crate::mem::linked::VisitMut;
use crate::mem::linked::VisitRef;
use crate::mem::linked::TraversalState;
use crate::mem::linked::UncondTravState;
use crate::mem::linked::visit_type::VisitType;

enum CurrentTraversalState<Node> {
    EnterNode(Index<Node>),
    ExitNode(Index<Node>),
}

use CurrentTraversalState::EnterNode;
use CurrentTraversalState::ExitNode;

/// Visits a linked data structure, with the following rules, attempting to
/// visit each node exactly once:
/// - The traversal starts by entering the start node.
/// - If node has a child, then `enter_node` is called with the node
/// and the traversal continues with the first child node, in a depth-first
/// manner.
/// - The traversal enters child nodes before sibling nodes.
/// - If the current node has no right element, then the node is exited.
///
/// For instance, for a full binary tree with height 2 (7 nodes),
/// enumerated in BFS order from A, will be traversed as:
/// enter A, enter B, enter D, exit D, enter E, exit E, exit B, enter C,
/// enter F, exit F, enter G, exit G, exit C, exit A.
pub fn visit_linked<Node, Arenas, State>(
    start_index: Index<Node>,
    start_state: State,
    arenas: &Arenas,
) -> ArenaResult<State>
where
    Node: Linked<Node, Arenas>,
    State: TraversalState<Node, Visit = VisitRef>,
{
    let final_state = uncond_visit_linked(start_index, start_state, arenas)?;
    Ok(final_state.state)
}

/// Similar to [visit_linked], but allows for modification during traversal.
/// Note that changing the links between nodes will also change the order of
/// traversal accordingly.
pub fn visit_linked_mut<Node, Arenas, State>(
    start_index: Index<Node>,
    start_state: State,
    arenas: &Arenas,
) -> ArenaResult<State>
where
    Node: Linked<Node, Arenas>,
    State: TraversalState<Node, Visit = VisitMut>,
{
    let final_state = uncond_visit_linked(start_index, start_state, arenas)?;
    Ok(final_state.state)
}

fn uncond_visit_linked<TravState, Node, Arenas>(
    start_index: Index<Node>,
    start_state: TravState,
    arenas: &Arenas,
) -> ArenaResult<TravState>
where
    Node: Linked<Node, Arenas>,
    TravState: TraversalState<Node, Output = ()>,
{
    let UncondTravState(output_state) = controlled_visit_linked(
        start_index,
        UncondTravState(start_state),
        arenas,
    )?;
    Ok(output_state)
}

fn enter_node<TravState, Node, Arenas>(
    node: Index<Node>,
    trav_state: &mut TravState,
    arenas: &Arenas,
) -> ArenaResult<ControlFlow<(), CurrentTraversalState<Node>>>
where
    Node: Linked<Node, Arenas>,
    TravState: TraversalState<Node, Output = bool>,
{
    let arena = <Node as Linked<_, _>>::child_arena(arenas);

    // Enter the current node
    let next_state =
        TravState::Visit::arena_map(arena, node.clone(), |cur_node: _| {
            let (success, cur_node) =
                TravState::Visit::with_ref(cur_node, |node| {
                    trav_state.enter_node(node)
                });
            // If requested, exit the current node and continue
            // traversal.
            if !success {
                return Continue(ExitNode(node.clone()));
            }
            // Get the first child, if there are any children
            let opt_first_child =
                <Node as Linked<_, _>>::get_children(cur_node.deref())
                    .map(Multiple::start)
                    .flatten();
            let Some(child) = opt_first_child else {
                // No children, so can only exit the node
                return Continue(ExitNode(node.clone()));
            };
            // Continue with the first child
            Continue(EnterNode(child))
        })?;
    trav_state.post_enter(node);
    Ok(next_state)
}

fn exit_node<TravState, Node, Arenas>(
    node: Index<Node>,
    trav_state: &mut TravState,
    arenas: &Arenas,
) -> ArenaResult<ControlFlow<(), CurrentTraversalState<Node>>>
where
    Node: Linked<Node, Arenas>,
    TravState: TraversalState<Node, Output = bool>,
{
    let arena = <Node as Linked<_, _>>::child_arena(arenas);

    // On exit, if there is a sibling, traverse to it, or else exit
    // the current node.
    let next_state =
        TravState::Visit::arena_map(arena, node.clone(), |cur_node: _| {
            // Check if we should exit early
            let (success, cur_node) =
                TravState::Visit::with_ref(cur_node, |node| {
                    trav_state.exit_node(node)
                });
            if !success {
                // Indicate to reenter the current node.
                return Continue(EnterNode(node.clone()));
            }
            // Get the next sibling node, if there is one.
            let opt_next = cur_node
                .deref()
                .get_sibling_chain()
                .next();
            // Try and move to the next sibling
            let Some(next) = opt_next else {
                // No next sibling, so can only move to the parent.
                // If there is no parent, then traversal finishes.
                let Some(parent) = cur_node.get_parent().clone() else {
                    return Break(());
                };
                return Continue(EnterNode(parent));
            };
            // Continue with the next sibling
            Continue(EnterNode(next))
        })?;
    trav_state.post_exit(node);
    Ok(next_state)
}

/// Like [visit_linked], but the traversal state callbacks `enter_node` and
/// `exit_node` can return booleans:
/// If `enter_node` returns false, then `exit_node` is called on the same node
/// immediately after `post_enter`.
/// If `exit_node` returns false, then `enter_node` is called on the same node
/// immediately after `post_exit`.
///
/// When both callbacks unconditionally return true, the behaviour is exactly
/// the same as [visit_linked].
pub fn controlled_visit_linked<TravState, Node, Arenas>(
    start_index: Index<Node>,
    start_state: TravState,
    arenas: &Arenas,
) -> ArenaResult<TravState>
where
    Node: Linked<Node, Arenas>,
    TravState: TraversalState<Node, Output = bool>,
{
    // The FSM state and traversal state take turns to update each other:
    // The FSM state dictates whether to traverse down (entering nodes)
    // or up (exiting nodes) in the tree.
    // The traversal state's output (the boolean) then dictates what the next
    // FSM state will be.
    let mut trav_state = start_state;
    let mut fsm_state = EnterNode(start_index);

    loop {
        let opt_next_fsm_state = match fsm_state {
            EnterNode(node) => enter_node(node, &mut trav_state, arenas)?,
            ExitNode(node) => exit_node(node, &mut trav_state, arenas)?,
        };
        match opt_next_fsm_state {
            Continue(next_fsm_state) => fsm_state = next_fsm_state,
            Break(()) => return Ok(trav_state),
        }
    }
}

/// Used to traverse two structures at the same time, possibly from two
/// different arenas.
/// The two nodes are entered and exited in tandem, assuming the callbacks
/// in `TravState` return true.
/// This is useful for implementing ordering between two arenas.
pub(super) fn controlled_visit_two_linked<TravState, Node, ArenasX, ArenasY>(
    start_index_x: Index<Node>,
    start_index_y: Index<Node>,
    start_state: TravState,
    arenas_x: &ArenasX,
    arenas_y: &ArenasY,
) -> ArenaResult<TravState>
where
    Node: Linked<Node, ArenasX>,
    Node: Linked<Node, ArenasY>,
    TravState: TraversalState<Node, Output = bool>,
{
    let mut trav_state = start_state;
    let mut fsm_state_x = EnterNode(start_index_x);
    let mut fsm_state_y = EnterNode(start_index_y);

    loop {
        let opt_next_fsm_state_x = match fsm_state_x {
            EnterNode(node_x) => enter_node(node_x, &mut trav_state, arenas_x)?,
            ExitNode(node_x) => exit_node(node_x, &mut trav_state, arenas_x)?,
        };
        let opt_next_fsm_state_y = match fsm_state_y {
            EnterNode(node_y) => enter_node(node_y, &mut trav_state, arenas_y)?,
            ExitNode(node_y) => exit_node(node_y, &mut trav_state, arenas_y)?,
        };

        match (opt_next_fsm_state_x, opt_next_fsm_state_y) {
            (Continue(next_fsm_state_x), Continue(next_fsm_state_y)) => {
                fsm_state_x = next_fsm_state_x;
                fsm_state_y = next_fsm_state_y;
            }
            (Break(()), _) | (_, Break(())) => return Ok(trav_state),
        }
    }
}
