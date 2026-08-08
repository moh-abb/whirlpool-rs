use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::linked::Linked;

impl<Arenas: PatternArenas> Linked<PatternNode, Arenas> for PatternNode {
    fn parent_arena(arenas: &Arenas) -> &impl Arena<PatternNode> {
        arenas.get_pattern_arena()
    }

    fn child_arena(arenas: &Arenas) -> &impl Arena<Self> {
        arenas.get_pattern_arena()
    }

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

    fn get_children(parent: &PatternNode) -> Option<&Multiple<Self>> {
        match &parent.pattern {
            Pattern::Cat(multiple)
            | Pattern::Seq(multiple)
            | Pattern::Stack(multiple)
            | Pattern::TimeCat { total_cycle_length: _, multiple }
            | Pattern::Arrange { total_cycle_length: _, multiple } => {
                Some(multiple)
            }
            Pattern::TimedStep(TimedStep(_, multiple)) => Some(multiple),
            Pattern::Note(_) | Pattern::Silence => None,
        }
    }

    fn get_mut_children(
        parent: &mut PatternNode,
    ) -> Option<&mut Multiple<Self>> {
        match &mut parent.pattern {
            Pattern::Cat(multiple)
            | Pattern::Seq(multiple)
            | Pattern::Stack(multiple)
            | Pattern::TimeCat { total_cycle_length: _, multiple }
            | Pattern::Arrange { total_cycle_length: _, multiple } => {
                Some(multiple)
            }
            Pattern::TimedStep(TimedStep(_, multiple)) => Some(multiple),
            Pattern::Note(_) | Pattern::Silence => None,
        }
    }
}
