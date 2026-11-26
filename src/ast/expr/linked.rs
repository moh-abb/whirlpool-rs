use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::adapter::ExprArenaAdapter;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::LinkedContainer;
use crate::mem::SharedArenaRef;
use crate::mem::linked::ChildMut;
use crate::mem::linked::ChildRef;
use crate::mem::linked::Linked;
use crate::mem::linked::LinkedNode;

/// Convenience trait for arenas of expressions.
pub trait ExprArenas: Arena<ExprNode> {}
impl<A: Arena<ExprNode>> ExprArenas for A {}

impl Linked<ExprNode> for ExprNode
where
    Self: ArenaItem + Sized,
    ExprNode: ArenaItem + Sized,
{
    fn get_parent(&self) -> &Option<Index<ExprNode>> {
        &self.0.parent
    }

    fn get_mut_parent(&mut self) -> &mut Option<Index<ExprNode>> {
        &mut self.0.parent
    }

    fn get_sibling_chain(&self) -> &Chain<Self> {
        &self.0.sibling_chain
    }

    fn get_mut_sibling_chain(&mut self) -> &mut Chain<Self> {
        &mut self.0.sibling_chain
    }

    fn get_children(parent: &ExprNode) -> Option<ChildRef<'_, Self>> {
        match &parent.0.item {
            Expr::Call { args, .. } | Expr::Array { args } => {
                Some(ChildRef::Multiple(args))
            }
            Expr::Lambda { expr, .. } => Some(ChildRef::Single(expr)),
            Expr::Unit(_) => None,
        }
    }

    fn get_mut_children(parent: &mut ExprNode) -> Option<ChildMut<'_, Self>> {
        match &mut parent.0.item {
            Expr::Call { args, .. } | Expr::Array { args } => {
                Some(ChildMut::Multiple(args))
            }
            Expr::Lambda { expr, .. } => Some(ChildMut::Single(expr)),
            Expr::Unit(_) => None,
        }
    }
}

/// Represents the error type when pushing back an element to an expression.
#[derive(derive_more::From, Debug, Clone)]
pub enum ExprPushBackError {
    ArenaErr(ArenaError),
    ExpectedFullAdapter,
}

impl<'r, A: ExprArenas> SharedArenaRef<'r, A> {
    pub fn push_child_adapter(
        &mut self,
        parent_index: Index<ExprNode>,
        mut child_adapter: ExprArenaAdapter<Self>,
    ) -> Result<(), ExprPushBackError> {
        let child_index = child_adapter
            .clone_index()?
            .ok_or(ExprPushBackError::ExpectedFullAdapter)?;

        // Link the pattern to the provided parent.
        let mut shared_arena_ref_1 = self.clone();
        let mut shared_arena_ref_2 = self.clone();
        LinkedContainer::push_child(
            &mut shared_arena_ref_1,
            &mut shared_arena_ref_2,
            parent_index,
            Cow::Indexed(child_index),
        )?;

        // Avoid dropping the newly-linked child
        child_adapter.take_index()?;

        Ok(())
    }

    pub fn push_to_adapter(
        &mut self,
        expr: Expr,
    ) -> ArenaResult<ExprArenaAdapter<Self>> {
        Ok(ExprArenaAdapter::new(
            self.push(ExprNode(LinkedNode::new(expr)))?,
            self.clone(),
        ))
    }
}
