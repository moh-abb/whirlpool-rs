use core::cmp::Ordering;

use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::linked::Linked;
use crate::mem::linked::TraversalState;
use crate::mem::linked::VisitRef;
use crate::mem::linked::visitor::controlled_visit_two_linked;

/// Used for comparing two linked structures, potentially in different arenas.
///
/// This is achieved by a lexicographic ordering. We attempt to enter both
/// nodes in tandem. When we leave node X before node Y, this means node Y is
/// deeper nested than node X, and so the structure at X is shorter (the
/// opposite when we leave node Y before node X).
///
/// One invariant is that when two structures are exactly equal, then we always
/// enter and leave the structures in tandem, and so both `copied_x` and
/// `copied_y` are [None] after either exiting or entering in tandem.
///
/// The other invariant is that when `result` is no longer [Ordering::Equal],
/// then we always indicate to exit the current node (in order to break early).
struct OrdHandler<Node, CmpF, CopyF> {
    cmp_func: CmpF,
    copy_func: CopyF,
    copied_x: Option<Node>,
    copied_y: Option<Node>,
    result: Ordering,
}

impl<Node, CmpF, CopyF> TraversalState<Node> for OrdHandler<Node, CmpF, CopyF>
where
    CopyF: FnMut(&Node) -> Node,
    CmpF: FnMut(&Node, &Node) -> Ordering,
{
    type Visit = VisitRef;
    type Output = bool;
    type Error = ArenaError;

    fn enter_node(
        &mut self,
        cur_node: &Node,
    ) -> Result<Self::Output, Self::Error> {
        // If the comparison has finished, return early:
        // false signals to exit and move to the parent
        if self.result != Ordering::Equal {
            return Ok(false);
        }
        // This assumes that in the typical case,
        // we always enter node X before node Y.
        if let Some(copied_x) = self.copied_x.as_ref() {
            // We have already entered X and are now entering Y
            let copied_y = (self.copy_func)(cur_node);
            // Compare X and Y and update the result
            let cmp = (self.cmp_func)(copied_x, &copied_y);
            self.result = cmp;
            // Clear the copied X because we might traverse downwards
            self.copied_x = None;
        } else {
            // We are about to enter X first
            debug_assert!(self.copied_y.is_none());
            let copied_x = (self.copy_func)(cur_node);
            self.copied_x = Some(copied_x);
        }
        Ok(self.result == Ordering::Equal)
    }

    fn exit_node(&mut self, _: &Node) -> Result<Self::Output, Self::Error> {
        // If the comparison has finished, return early:
        // true signals to move to the parent (false would mean we reenter).
        if self.result != Ordering::Equal {
            return Ok(true);
        }
        // By the invariant, we should never have both `copied_x` and
        // `copied_y` be `Some`.
        debug_assert!(self.copied_x.is_none() || self.copied_y.is_none());
        // This assumes that in the typical case,
        // we always enter node X before node Y.
        if self.copied_x.is_some() {
            // We are exiting Y even though X has not exited yet.
            // This means that the structure at X is longer than Y by our
            // lexicographically ordering, so X > Y
            self.result = Ordering::Greater;
        } else if self.copied_y.is_some() {
            // Like above, we are currently exiting X even though Y has not
            // exited yet, and so X < Y
            self.result = Ordering::Less;
        }
        Ok(true)
    }
}

/// Compares two linked data structures using a provided comparator for
/// individual elements. This provides a kind of lexicographic ordering
/// between the two structures.
/// Note that this also uses a cloning function in order to save a temporary
/// copy of the `Node` to use during traversal.
pub fn cmp_linked<Node, ArenasX, ArenasY>(
    start_index_x: Index<Node>,
    start_index_y: Index<Node>,
    arenas_x: &ArenasX,
    arenas_y: &ArenasY,
    cmp_func: impl FnMut(&Node, &Node) -> Ordering,
    copy_func: impl FnMut(&Node) -> Node,
) -> ArenaResult<Ordering>
where
    Node: Linked<Node, ArenasX>,
    Node: Linked<Node, ArenasY>,
{
    let mut state = OrdHandler {
        cmp_func,
        copy_func,
        copied_x: None,
        copied_y: None,
        result: Ordering::Equal,
    };
    controlled_visit_two_linked(
        start_index_x,
        start_index_y,
        &mut state,
        arenas_x,
        arenas_y,
    )?;
    Ok(state.result)
}
