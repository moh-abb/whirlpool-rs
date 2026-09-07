use crate::ast::CycleTime;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;

pub mod arbitrary;
pub mod arenas;
pub mod clone;
pub mod cmp;
pub mod drop;
pub mod format;
pub mod linked;
pub mod note;
pub mod parser;

/// Represents a [Pattern] played with a given duration.
///
/// To allow for recursion, the inner pattern is a [Multiple]. However, this
/// should only have exactly one child element.
///
/// To maintain the type-level invariants, this should only be a child of a
/// pattern which expects [TimedStep] elements; i.e. [Pattern::TimeCat] or
/// [Pattern::Arrange].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimedStep(pub CycleTime, pub Multiple<PatternNode>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pattern {
    Cat(Multiple<PatternNode>),
    Seq(Multiple<PatternNode>),
    Stack(Multiple<PatternNode>),
    TimeCat { total_cycle_length: CycleTime, multiple: Multiple<PatternNode> },
    Arrange { total_cycle_length: CycleTime, multiple: Multiple<PatternNode> },
    TimedStep(TimedStep),
    Note(note::NoteUnit),
    Silence,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatternNode {
    pub parent: Option<Index<Self>>,
    pub sibling_chain: Chain<Self>,
    pub pattern: Pattern,
}

impl PatternNode {
    pub const fn new(pattern: Pattern) -> Self {
        Self { parent: None, sibling_chain: Chain::new(), pattern }
    }
}

const fn pattern_discriminant(pattern: &Pattern) -> u8 {
    match pattern {
        Pattern::Cat(_) => 1,
        Pattern::Seq(_) => 2,
        Pattern::Stack(_) => 3,
        Pattern::TimeCat { .. } => 4,
        Pattern::Arrange { .. } => 5,
        Pattern::TimedStep(_) => 6,
        Pattern::Note(_) => 7,
        Pattern::Silence => 8,
    }
}
