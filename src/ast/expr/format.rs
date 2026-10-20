use core::fmt;

use crate::ast::display::AstDisplay;
use crate::ast::display::AstDisplayFlags;
use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::ident::DisplayIdentError;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::ArenaError;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::traversal_state::TraversalState;
use crate::mem::linked::visit_type::VisitRef;
use crate::mem::linked::visitor::visit_linked;

pub struct ExprDisplayAdapter<'a, Arenas> {
    index: Index<ExprNode>,
    arenas: &'a Arenas,
}

impl<'a, Arenas: ExprArenas> ExprDisplayAdapter<'a, Arenas> {
    #[allow(unused)]
    pub fn new(index: Index<ExprNode>, arenas: &'a Arenas) -> Self {
        Self { index, arenas }
    }
}

struct DisplayTraversalState<'a, 'b> {
    formatter: &'a mut fmt::Formatter<'b>,
    flags: &'a AstDisplayFlags,
}

#[derive(derive_more::From, Debug)]
pub enum DisplayError {
    ArenaErr(ArenaError),
    FormatErr(fmt::Error),
    DisplayIdentErr(DisplayIdentError),
}

impl<'a, 'b> TraversalState<ExprNode> for DisplayTraversalState<'a, 'b> {
    type Visit = VisitRef;
    type Output = ();
    type Error = DisplayError;

    fn enter_node(
        &mut self,
        cur_node: &ExprNode,
    ) -> Result<Self::Output, Self::Error> {
        let f = &mut self.formatter;
        if cur_node.sibling_chain.prev().is_some() {
            write!(f, ",")?;
        }
        match &cur_node.expr {
            Expr::Call { name, .. } => {
                // TODO: Display method calls as taking receiver: `foo.bar(x)`.
                name.fmt_ast(f, self.flags)?;
                write!(f, "(")?;
            }
            Expr::Lambda { param, .. } => {
                param.fmt_ast(f, self.flags)?;
                write!(f, "=>")?;
                // The expression will be displayed on traversal.
            }
            Expr::Array { .. } => {
                write!(f, "[")?;
            }
            Expr::Unit(ExprUnit::Ident { name, .. }) => {
                name.fmt_ast(f, self.flags)?;
            }
            Expr::Unit(ExprUnit::Number(cycle_num)) => {
                write!(f, "{cycle_num}")?;
            }
            Expr::Unit(ExprUnit::String(_)) => {
                write!(f, "\"\"")?;
            }
        }
        Ok(())
    }

    fn exit_node(
        &mut self,
        cur_node: &ExprNode,
    ) -> Result<Self::Output, Self::Error> {
        let f = &mut self.formatter;
        match &cur_node.expr {
            Expr::Call { name, .. } => {
                // For now, display both methods and normal calls identically.
                name.fmt_ast(f, self.flags)?;
                write!(f, ")")?;
            }
            Expr::Lambda { .. } => (),
            Expr::Array { .. } => {
                write!(f, "]")?;
            }
            Expr::Unit(_) => (),
        }
        Ok(())
    }
}

fn visit_and_display_pattern<'a, 'b>(
    index: Index<ExprNode>,
    arenas: &impl ExprArenas,
    formatter: &'a mut fmt::Formatter<'b>,
    flags: &'a AstDisplayFlags,
) -> Result<(), DisplayError> {
    let mut state = DisplayTraversalState { formatter, flags };
    visit_linked(index, &mut state, arenas)?;
    Ok(())
}

impl<'a, Arenas: ExprArenas> fmt::Display for ExprDisplayAdapter<'a, Arenas> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_unwrap(visit_and_display_pattern(
            self.index.clone(),
            self.arenas,
            f,
            &AstDisplayFlags::default(),
        ));
        Ok(())
    }
}

impl<'a, Arenas: ExprArenas> AstDisplay for ExprDisplayAdapter<'a, Arenas> {
    type Error = DisplayError;
}
