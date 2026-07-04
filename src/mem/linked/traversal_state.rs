use crate::mem::Index;
use crate::mem::linked::visit_type::VisitType;

/// The traversal state (with a strategy that allows for potential control
/// over movement within the linked structure).
///
/// `post_enter` and `post_exit` are called with the node index after the
/// node is entered or exited, respectively.
///
/// When `Output` is the unit, this allows for normal uninterrupted traversal
/// (with the boolean return values detailed below always true), such that
/// all nodes should be visited exactly once.
///
/// When `Output` is a boolean, this allows for interruption and repetition of
/// traversal via callbacks that return boolean values:
/// - for `enter_node`: true indicates to visit the current node's children,
/// otherwise the children are skipped and the next action will be that the
/// current node is exited.
/// - for `exit_node`: true indicates to move to the parent after exiting the
/// node, otherwise the traversal repeats, so that the next action will be that
/// the current node is entered again.
pub trait TraversalState<Node>
where
    Self::Visit: VisitType,
{
    type Visit;
    type Output;

    fn enter_node(
        &mut self,
        cur_node: <Self::Visit as VisitType>::Ref<'_, Node>,
    ) -> Self::Output;

    fn exit_node(
        &mut self,
        cur_node: <Self::Visit as VisitType>::Ref<'_, Node>,
    ) -> Self::Output;

    fn post_enter(&mut self, index: Index<Node>) {}

    fn post_exit(&mut self, index: Index<Node>) {}
}

pub struct UncondTravState<TravState>(pub TravState);

impl<Node, TravState> TraversalState<Node> for UncondTravState<TravState>
where
    TravState: TraversalState<Node, Output = ()>,
{
    type Visit = TravState::Visit;
    type Output = bool;

    fn enter_node(
        &mut self,
        cur_node: <Self::Visit as VisitType>::Ref<'_, Node>,
    ) -> Self::Output {
        self.0.enter_node(cur_node);
        true
    }

    fn exit_node(
        &mut self,
        cur_node: <Self::Visit as VisitType>::Ref<'_, Node>,
    ) -> Self::Output {
        self.0.exit_node(cur_node);
        true
    }

    fn post_enter(&mut self, index: Index<Node>) {
        self.0.post_enter(index)
    }

    fn post_exit(&mut self, index: Index<Node>) {
        self.0.post_exit(index)
    }
}
