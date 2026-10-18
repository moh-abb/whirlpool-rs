use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::linked::ChildMut;
use crate::mem::linked::ChildRef;
use crate::mem::linked::Linked;

impl<Arenas: PatternArenas> Linked<PatternNode, Arenas, Arenas>
    for PatternNode
{
    fn get_parent(&self) -> &Option<Index<PatternNode>> {
        &self.parent
    }

    fn get_mut_parent(&mut self) -> &mut Option<Index<PatternNode>> {
        &mut self.parent
    }

    fn get_sibling_chain(&self) -> &Chain<Self> {
        &self.sibling_chain
    }

    fn get_mut_sibling_chain(&mut self) -> &mut Chain<Self> {
        &mut self.sibling_chain
    }

    fn get_children(parent: &PatternNode) -> Option<ChildRef<'_, PatternNode>> {
        match &parent.pattern {
            Pattern::Cat(multiple)
            | Pattern::Seq(multiple)
            | Pattern::Stack(multiple)
            | Pattern::TimeCat { multiple, .. }
            | Pattern::Arrange { multiple, .. } => {
                Some(ChildRef::Multiple(multiple))
            }
            Pattern::TimedStep(TimedStep(_, opt_child)) => {
                Some(ChildRef::Single(opt_child))
            }
            Pattern::Note(_) | Pattern::Silence => None,
        }
    }

    fn get_mut_children(
        parent: &mut PatternNode,
    ) -> Option<ChildMut<'_, Self>> {
        match &mut parent.pattern {
            Pattern::Cat(multiple)
            | Pattern::Seq(multiple)
            | Pattern::Stack(multiple)
            | Pattern::TimeCat { multiple, .. }
            | Pattern::Arrange { multiple, .. } => {
                Some(ChildMut::Multiple(multiple))
            }
            Pattern::TimedStep(TimedStep(_, opt_child)) => {
                Some(ChildMut::Single(opt_child))
            }
            Pattern::Note(_) | Pattern::Silence => None,
        }
    }
}
