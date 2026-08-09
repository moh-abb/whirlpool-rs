use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::ArenaMocker;
use crate::mem::GrowableArena;

#[derive(Debug)]
pub struct GrowableArenas(GrowableArena<PatternNode>);

impl Default for GrowableArenas {
    fn default() -> Self {
        Self(GrowableArena::new())
    }
}

impl PatternArenas for GrowableArenas {
    fn get_pattern_arena(&self) -> &impl Arena<PatternNode> {
        &self.0
    }
}

#[derive(Debug)]
pub struct PatternArenaMocker(pub ArenaMocker<PatternNode>);

impl PatternArenaMocker {
    pub fn new() -> Self {
        Self(ArenaMocker::new())
    }
}

impl PatternArenas for PatternArenaMocker {
    fn get_pattern_arena(&self) -> &impl Arena<PatternNode> {
        &self.0
    }
}
