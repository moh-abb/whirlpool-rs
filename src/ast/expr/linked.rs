use crate::ast::expr::PatternExpr;
use crate::ast::expr::PatternExprNode;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::linked::ChildMut;
use crate::mem::linked::ChildRef;
use crate::mem::linked::Linked;

impl<Arenas> Linked<PatternExprNode, Arenas, Arenas> for PatternExprNode
where
    Self: ArenaItem + Sized,
    PatternExprNode: ArenaItem + Sized,
    Arenas: Arena<Self>,
{
    fn get_parent(&self) -> &Option<Index<PatternExprNode>> {
        &self.parent
    }

    fn get_mut_parent(&mut self) -> &mut Option<Index<PatternExprNode>> {
        &mut self.parent
    }

    fn get_sibling_chain(&self) -> &Chain<Self> {
        &self.sibling_chain
    }

    fn get_mut_sibling_chain(&mut self) -> &mut Chain<Self> {
        &mut self.sibling_chain
    }

    fn get_children(parent: &PatternExprNode) -> Option<ChildRef<'_, Self>> {
        match &parent.expr {
            PatternExpr::Call { args, .. } | PatternExpr::Array { args } => {
                Some(ChildRef::Multiple(args))
            }
            PatternExpr::Lambda { expr, .. } => Some(ChildRef::Single(expr)),
            PatternExpr::Unit(_) => None,
        }
    }

    fn get_mut_children(
        parent: &mut PatternExprNode,
    ) -> Option<ChildMut<'_, Self>> {
        match &mut parent.expr {
            PatternExpr::Call { args, .. } | PatternExpr::Array { args } => {
                Some(ChildMut::Multiple(args))
            }
            PatternExpr::Lambda { expr, .. } => Some(ChildMut::Single(expr)),
            PatternExpr::Unit(_) => None,
        }
    }
}
