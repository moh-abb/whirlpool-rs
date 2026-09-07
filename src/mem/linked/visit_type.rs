//! Used for indicating whether traversal of a linked data structure will occur
//! mutably or immutably.

use core::ops::ControlFlow;
use core::ops::ControlFlow::Break;
use core::ops::ControlFlow::Continue;
use core::ops::Deref;

use private::*;

use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::linked::Linked;
use crate::mem::linked::traversal_state::TraversalState;

/// Indicates that visiting the data structure will occur immutably.
pub struct VisitRef;
/// Indicates that visiting the data structure will occur mutably.
pub struct VisitMut;

#[macro_use]
pub(super) mod private {
    use super::*;

    pub enum CurrentTraversalState<Node> {
        EnterNode(Index<Node>),
        ExitNode(Index<Node>),
    }

    pub use CurrentTraversalState::EnterNode;
    pub use CurrentTraversalState::ExitNode;

    macro_rules! enter_node_func {
        (block $node:ident $state:ident $arena:ident $body:block) => {
            fn enter_node<'a, Node, NodeArena, TravState>(
                $node: Index<Node>,
                $state: &mut TravState,
                $arena: &mut Self::Ref<'a, NodeArena>,
            ) -> Result<
                ControlFlow<(), CurrentTraversalState<Node>>,
                TravState::Error,
            >
            where
                Node: ArenaItem,
                NodeArena: Arena<Node> + 'a,
                Node: Linked<Node, NodeArena, NodeArena>,
                TravState: TraversalState<Node, Visit = Self, Output = bool>
                $body
        };
        (head) => {
            enter_node_func!(block _node _state _arena { Ok(Break(())) });
        };
        (body $arena_map_func:ident) => {
            enter_node_func!(block node state arena {
                let continue_enter = |node: Index<Node>| {
                    Result::<_, TravState::Error>::Ok(Continue(EnterNode(node)))
                };
                let continue_exit = |node: Index<Node>| {
                    Result::<_, TravState::Error>::Ok(Continue(ExitNode(node)))
                };
                // Enter the current node
                let opt_next_state =
                    arena.$arena_map_func(node.clone(), |cur_node| {
                        let success = state.enter_node(cur_node)?;
                        // If requested, exit the current node and continue
                        // traversal.
                        if !success {
                            return continue_exit(node.clone());
                        }
                        // Get the first child, if there are any children
                        let opt_first_child =
                            <Node as Linked<_, _, _>>::get_children(
                                cur_node.deref(),
                            )
                            .map(Multiple::start)
                            .flatten();
                        let Some(child) = opt_first_child else {
                            // No children, so can only exit the node
                            return continue_exit(node.clone());
                        };
                        // Continue with the first child
                        continue_enter(child)
                    });
                let next_state = opt_next_state??;
                state.post_enter(node, *arena)?;
                Ok(next_state)
            });
        };
    }

    macro_rules! exit_node_func {
        (block $node:ident $state:ident $arena:ident $body:block) => {
            fn exit_node<'a, Node, NodeArena, TravState>(
                $node: Index<Node>,
                $state: &mut TravState,
                $arena: &mut Self::Ref<'a, NodeArena>,
            ) -> Result<
                ControlFlow<(), CurrentTraversalState<Node>>,
                TravState::Error,
            >
            where
                Node: ArenaItem,
                NodeArena: Arena<Node> + 'a,
                Node: Linked<Node, NodeArena, NodeArena>,
                TravState: TraversalState<Node, Visit = Self, Output = bool>
                $body
        };
        (head) => {
            exit_node_func!(block _node _state _ident { Ok(Break(())) });
        };
        (body $arena_map_func:ident) => {
            exit_node_func!(block node state arena {
                let continue_enter = |node: Index<Node>| {
                    Result::<_, TravState::Error>::Ok(Continue(EnterNode(node)))
                };
                let continue_exit = |node: Index<Node>| {
                    Result::<_, TravState::Error>::Ok(Continue(ExitNode(node)))
                };
                // On exit, if there is a sibling, traverse to it, or else exit
                // the current node.
                // Check if we should exit early first.
                let opt_next_state =
                    arena.$arena_map_func(node.clone(), |cur_node| {
                        let success = state.exit_node(cur_node)?;
                        if !success {
                            // Indicate to reenter the current node.
                            return continue_enter(node.clone());
                        }
                        // Get the next sibling node, if there is one.
                        let opt_next = cur_node.get_sibling_chain().next();
                        // Try and move to the next sibling
                        let Some(next) = opt_next else {
                            // No next sibling, so can only move to exit the
                            // parent. If there is no parent,
                            // then traversal finishes.
                            let Some(parent) = cur_node.get_parent().clone()
                            else {
                                return Ok(Break(()));
                            };
                            return continue_exit(parent);
                        };
                        // Continue with the next sibling
                        continue_enter(next)
                    });
                let next_state = opt_next_state??;
                state.post_exit(node, *arena)?;
                Ok(next_state)
            });
        };
    }

    pub trait SealedVisitType {
        type Ref<'a, Item: 'a>;

        enter_node_func!(head);
        exit_node_func!(head);
    }
}

impl VisitType for VisitRef {}
impl VisitType for VisitMut {}
pub trait VisitType: SealedVisitType {}

impl SealedVisitType for VisitRef {
    type Ref<'a, Item: 'a> = &'a Item;

    enter_node_func!(body map);
    exit_node_func!(body map);
}

impl SealedVisitType for VisitMut {
    type Ref<'a, Item: 'a> = &'a mut Item;

    enter_node_func!(body map_mut);
    exit_node_func!(body map_mut);
}
