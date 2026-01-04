use crate::arena::index::Index;
use crate::ast::note::NoteUnit;
use crate::ast::pattern::Pattern;

#[allow(unused)]
#[derive(Debug)]
pub enum ChainedPatternType {
    Cat,
    Seq,
    Stack,
}

#[allow(unused)]
#[derive(Debug)]
pub enum NewPatternType {
    ChainedPattern { pattern_type: ChainedPatternType },
    TimeCat,
    NoteUnit { unit: NoteUnit },
    Silence,
}

#[allow(unused)]
#[derive(Debug)]
pub enum PatternActionType {
    CreatePattern {
        pattern_type: NewPatternType,
    },
    DeletePattern,
    CopyItemInPatternChain {
        offset: u16,
    },
    RemoveItemInPatternChain {
        offset: u16,
    },
    NewUnitInPatternChain {
        unit: NoteUnit,
        offset: u16,
    },
    NewSilenceInPatternChain {
        offset: u16,
    },
    ConvertChainedPatternType {
        from_type: ChainedPatternType,
        to_type: ChainedPatternType,
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct PatternAction {
    in_pattern: Index<Pattern>,
    action_type: PatternActionType,
}
