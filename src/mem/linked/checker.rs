#![cfg(test)]

use core::fmt::Debug;
use std::collections::BTreeSet;

use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::Index;
use crate::mem::linked::Linked;
use crate::mem::linked::traversal_state::TraversalState;
use crate::mem::linked::visit_type::VisitRef;
use crate::mem::linked::visitor::visit_linked;

struct CycleFinderState<Node> {
    visited: BTreeSet<Node>,
}

impl<Node> TraversalState<Node> for CycleFinderState<Node>
where
    Node: ArenaItem + Clone + Debug,
{
    type Visit = VisitRef;
    type Output = ();
    type Error = ArenaError;

    fn enter_node(
        &mut self,
        cur_node: &Node,
    ) -> Result<Self::Output, Self::Error> {
        println!("Entering node {cur_node:?}");
        if self.visited.contains(cur_node) {
            panic!(
                "Node has already been visited, all visited nodes: {:?}",
                &self.visited
            );
        }
        self.visited.insert(cur_node.clone());
        Ok(())
    }

    fn exit_node(
        &mut self,
        cur_node: &Node,
    ) -> Result<Self::Output, Self::Error> {
        println!("Exiting node {cur_node:?}");
        Ok(())
    }

    fn post_enter(
        &mut self,
        index: Index<Node>,
        _: &impl Arena<Node>,
    ) -> Result<(), Self::Error> {
        println!("Entered {index:?}");
        Ok(())
    }

    fn post_exit(
        &mut self,
        index: Index<Node>,
        _: &impl Arena<Node>,
    ) -> Result<(), Self::Error> {
        println!("Exited {index:?}");
        Ok(())
    }
}

pub fn check_acyclic<Node, Arenas>(start_index: Index<Node>, arenas: &Arenas)
where
    Arenas: Arena<Node>,
    Node: Linked<Node, Arenas, Arenas> + Clone + Debug,
{
    let mut state = CycleFinderState { visited: BTreeSet::new() };
    visit_linked(start_index, &mut state, arenas).unwrap();
}
