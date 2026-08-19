use core::ops::ControlFlow::Break;
use core::ops::ControlFlow::Continue;

use crate::mem::Arena;
use crate::mem::Index;
use crate::mem::linked::Linked;
use crate::mem::linked::TraversalState;
use crate::mem::linked::VisitMut;
use crate::mem::linked::VisitRef;
use crate::mem::linked::traversal_state::UncondTravState;
use crate::mem::linked::visit_type::CurrentTraversalState::EnterNode;
use crate::mem::linked::visit_type::CurrentTraversalState::ExitNode;
use crate::mem::linked::visit_type::private::SealedVisitType;

#[cfg(debug_assertions)]
const MAX_ITERATION_COUNT: usize = 100_000;

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
pub fn visit_linked<TravState, Node, NodeArena>(
    start_index: Index<Node>,
    start_state: &mut TravState,
    arena: &NodeArena,
) -> Result<(), TravState::Error>
where
    NodeArena: Arena<Node>,
    Node: Linked<Node, NodeArena, NodeArena>,
    TravState: TraversalState<Node, Visit = VisitRef, Output = ()>,
{
    uncond_visit_linked(start_index, start_state, arena)
}

/// Similar to [visit_linked], but allows for modification during traversal.
/// Note that changing the links between nodes will also change the order of
/// traversal accordingly.
pub fn visit_linked_mut<TravState, Node, NodeArena>(
    start_index: Index<Node>,
    state: &mut TravState,
    arena: &mut NodeArena,
) -> Result<(), TravState::Error>
where
    NodeArena: Arena<Node>,
    Node: Linked<Node, NodeArena, NodeArena>,
    TravState: TraversalState<Node, Visit = VisitMut, Output = ()>,
{
    uncond_visit_linked(start_index, state, arena)
}

fn uncond_visit_linked<'a, 's, TravState, Node, NodeArena>(
    start_index: Index<Node>,
    state: &'s mut TravState,
    arena: <TravState::Visit as SealedVisitType>::Ref<'a, NodeArena>,
) -> Result<(), TravState::Error>
where
    NodeArena: Arena<Node>,
    Node: Linked<Node, NodeArena, NodeArena> + 'a,
    TravState: TraversalState<Node, Output = ()>,
{
    let mut uncond_state = UncondTravState(state);
    controlled_visit_linked(start_index, &mut uncond_state, arena)
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
pub fn controlled_visit_linked<'a, TravState, Node, NodeArena>(
    start_index: Index<Node>,
    state: &mut TravState,
    mut arena: <TravState::Visit as SealedVisitType>::Ref<'a, NodeArena>,
) -> Result<(), TravState::Error>
where
    NodeArena: Arena<Node>,
    Node: Linked<Node, NodeArena, NodeArena> + 'a,
    TravState: TraversalState<Node, Output = bool>,
{
    // The FSM state and traversal state take turns to update each other:
    // The FSM state dictates whether to traverse down (entering nodes)
    // or up (exiting nodes) in the tree.
    // The traversal state's output (the boolean) then dictates what the next
    // FSM state will be.
    let mut fsm_state = EnterNode(start_index);

    #[cfg(debug_assertions)]
    let mut cur_iters = 0;

    #[cfg(debug_assertions)]
    let mut update_iter = || {
        if cur_iters > MAX_ITERATION_COUNT {
            panic!("Reached max iteration count")
        }
        cur_iters += 1;
    };

    loop {
        let opt_next_fsm_state = match fsm_state {
            EnterNode(node) => {
                TravState::Visit::enter_node(node, state, &mut arena)
            }
            ExitNode(node) => {
                TravState::Visit::exit_node(node, state, &mut arena)
            }
        };

        match opt_next_fsm_state? {
            Continue(next_fsm_state) => fsm_state = next_fsm_state,
            Break(()) => return Ok(()),
        }

        #[cfg(debug_assertions)]
        update_iter()
    }
}

/// Used to traverse two structures at the same time, possibly from two
/// different arenas.
/// The two nodes are entered and exited in tandem, assuming the callbacks
/// in `TravState` return true.
/// This is useful for implementing ordering between two arenas.
pub(super) fn controlled_visit_two_linked<'a, TravState, Node, AX, AY>(
    start_index_x: Index<Node>,
    start_index_y: Index<Node>,
    state: &mut TravState,
    mut arenas_x: <TravState::Visit as SealedVisitType>::Ref<'a, AX>,
    mut arenas_y: <TravState::Visit as SealedVisitType>::Ref<'a, AY>,
) -> Result<(), TravState::Error>
where
    AX: Arena<Node>,
    AY: Arena<Node>,
    Node: Linked<Node, AX, AX> + 'a,
    Node: Linked<Node, AY, AY> + 'a,
    TravState: TraversalState<Node, Output = bool>,
{
    let mut fsm_state_x = EnterNode(start_index_x);
    let mut fsm_state_y = EnterNode(start_index_y);

    #[cfg(debug_assertions)]
    let mut cur_iters = 0;

    #[cfg(debug_assertions)]
    let mut update_iter = || {
        if cur_iters > MAX_ITERATION_COUNT {
            panic!("Reached max iteration count")
        }
        cur_iters += 1;
    };

    loop {
        let opt_next_fsm_state_x = match fsm_state_x {
            EnterNode(node_x) => {
                TravState::Visit::enter_node(node_x, state, &mut arenas_x)
            }
            ExitNode(node_x) => {
                TravState::Visit::exit_node(node_x, state, &mut arenas_x)
            }
        };

        let opt_next_fsm_state_y = match fsm_state_y {
            EnterNode(node_y) => {
                TravState::Visit::enter_node(node_y, state, &mut arenas_y)
            }
            ExitNode(node_y) => {
                TravState::Visit::exit_node(node_y, state, &mut arenas_y)
            }
        };

        match (opt_next_fsm_state_x?, opt_next_fsm_state_y?) {
            (Continue(next_fsm_state_x), Continue(next_fsm_state_y)) => {
                fsm_state_x = next_fsm_state_x;
                fsm_state_y = next_fsm_state_y;
            }
            (Break(()), _) | (_, Break(())) => return Ok(()),
        }

        #[cfg(debug_assertions)]
        update_iter()
    }
}
