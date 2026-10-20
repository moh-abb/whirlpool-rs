use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::linked::ChildMut;
use crate::mem::linked::ChildRef;
use crate::mem::linked::Linked;

impl<Arenas> Linked<ExprNode, Arenas, Arenas> for ExprNode
where
    Self: ArenaItem + Sized,
    ExprNode: ArenaItem + Sized,
    Arenas: Arena<Self>,
{
    fn get_parent(&self) -> &Option<Index<ExprNode>> {
        &self.parent
    }

    fn get_mut_parent(&mut self) -> &mut Option<Index<ExprNode>> {
        &mut self.parent
    }

    fn get_sibling_chain(&self) -> &Chain<Self> {
        &self.sibling_chain
    }

    fn get_mut_sibling_chain(&mut self) -> &mut Chain<Self> {
        &mut self.sibling_chain
    }

    fn get_children(parent: &ExprNode) -> Option<ChildRef<'_, Self>> {
        match &parent.expr {
            Expr::Call { args, .. } | Expr::Array { args } => {
                Some(ChildRef::Multiple(args))
            }
            Expr::Lambda { expr, .. } => Some(ChildRef::Single(expr)),
            Expr::Unit(_) => None,
        }
    }

    fn get_mut_children(parent: &mut ExprNode) -> Option<ChildMut<'_, Self>> {
        match &mut parent.expr {
            Expr::Call { args, .. } | Expr::Array { args } => {
                Some(ChildMut::Multiple(args))
            }
            Expr::Lambda { expr, .. } => Some(ChildMut::Single(expr)),
            Expr::Unit(_) => None,
        }
    }
}
