use core::fmt;

use crate::ast::display::AstDisplay;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::ArenaError;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::traversal_state::TraversalState;
use crate::mem::linked::visit_type::VisitRef;
use crate::mem::linked::visitor::visit_linked;

pub struct PatternDisplayAdapter<'a, Arenas> {
    index: Index<PatternNode>,
    arenas: &'a Arenas,
}

impl<'a, Arenas: PatternArenas> PatternDisplayAdapter<'a, Arenas> {
    #[allow(unused)]
    pub fn new(index: Index<PatternNode>, arenas: &'a Arenas) -> Self {
        Self { index, arenas }
    }
}

struct DisplayTraversalState<'a, 'b> {
    formatter: &'a mut fmt::Formatter<'b>,
}

#[derive(derive_more::From, Debug)]
pub enum DisplayError {
    ArenaErr(ArenaError),
    FormatErr(fmt::Error),
}

impl<'a, 'b> TraversalState<PatternNode> for DisplayTraversalState<'a, 'b> {
    type Visit = VisitRef;
    type Output = ();
    type Error = DisplayError;

    fn enter_node(
        &mut self,
        cur_node: &PatternNode,
    ) -> Result<Self::Output, Self::Error> {
        let f = &mut self.formatter;
        if cur_node.sibling_chain.prev().is_some() {
            write!(f, ", ")?;
        }
        match cur_node.pattern {
            Pattern::Cat(_) => write!(f, "cat("),
            Pattern::Seq(_) => write!(f, "seq("),
            Pattern::Stack(_) => write!(f, "stack("),
            Pattern::TimeCat { .. } => write!(f, "timeCat("),
            Pattern::Arrange { .. } => write!(f, "arrange("),
            Pattern::TimedStep(ref timed_step) => {
                write!(f, "[{}, ", timed_step.0)
            }
            Pattern::Note(note_unit) => write!(f, "{}", note_unit),
            Pattern::Silence => write!(f, "~"),
        }?;
        Ok(())
    }

    fn exit_node(
        &mut self,
        cur_node: &PatternNode,
    ) -> Result<Self::Output, Self::Error> {
        let f = &mut self.formatter;
        match cur_node.pattern {
            Pattern::Cat(_)
            | Pattern::Seq(_)
            | Pattern::Stack(_)
            | Pattern::TimeCat { .. }
            | Pattern::Arrange { .. } => write!(f, ")"),
            Pattern::TimedStep(_) => write!(f, "]"),
            Pattern::Note(_) | Pattern::Silence => Ok(()),
        }?;
        Ok(())
    }
}

fn visit_and_display_pattern<'a, 'b>(
    index: Index<PatternNode>,
    arenas: &impl PatternArenas,
    formatter: &'a mut fmt::Formatter<'b>,
) -> Result<(), DisplayError> {
    let mut state = DisplayTraversalState { formatter };
    visit_linked(index, &mut state, arenas)?;
    Ok(())
}

impl<'a, Arenas: PatternArenas> fmt::Display
    for PatternDisplayAdapter<'a, Arenas>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_unwrap(visit_and_display_pattern(
            self.index.clone(),
            self.arenas,
            f,
        ));
        Ok(())
    }
}

impl<'a, Arenas: PatternArenas> AstDisplay
    for PatternDisplayAdapter<'a, Arenas>
{
    type Error = DisplayError;
}
