use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::ArenaMocker;
use crate::mem::Chain;
use crate::mem::GrowableArena;

#[derive(Debug)]
pub struct GrowableArenas(
    GrowableArena<Pattern>,
    GrowableArena<Chain<Pattern>>,
    GrowableArena<TimedStep>,
    GrowableArena<Chain<TimedStep>>,
);

impl Default for GrowableArenas {
    fn default() -> Self {
        new_growable_arena_tuple()
    }
}

impl PatternArenas for GrowableArenas {
    fn get_pattern_arena(&self) -> &impl Arena<Pattern> {
        &self.0
    }

    fn get_pattern_chain_arena(&self) -> &impl Arena<Chain<Pattern>> {
        &self.1
    }

    fn get_timed_step_arena(&self) -> &impl Arena<TimedStep> {
        &self.2
    }

    fn get_timed_step_chain_arena(&self) -> &impl Arena<Chain<TimedStep>> {
        &self.3
    }
}

fn new_growable_arena_tuple() -> GrowableArenas {
    GrowableArenas(
        GrowableArena::new(),
        GrowableArena::new(),
        GrowableArena::new(),
        GrowableArena::new(),
    )
}

pub fn get_arena_sizes(arenas: &impl PatternArenas) -> impl Fn() -> [usize; 4] {
    let arena_tuple = (
        arenas.get_pattern_arena(),
        arenas.get_pattern_chain_arena(),
        arenas.get_timed_step_arena(),
        arenas.get_timed_step_chain_arena(),
    );
    || {
        [
            arena_tuple.0.size(),
            arena_tuple.1.size(),
            arena_tuple.2.size(),
            arena_tuple.3.size(),
        ]
    }
}

#[derive(Debug)]
pub struct PatternArenaMockers(
    pub ArenaMocker<Pattern>,
    pub ArenaMocker<Chain<Pattern>>,
    pub ArenaMocker<TimedStep>,
    pub ArenaMocker<Chain<TimedStep>>,
);

impl PatternArenaMockers {
    pub fn new() -> Self {
        Self(
            ArenaMocker::new(),
            ArenaMocker::new(),
            ArenaMocker::new(),
            ArenaMocker::new(),
        )
    }
}

impl PatternArenas for PatternArenaMockers {
    fn get_pattern_arena(&self) -> &impl Arena<Pattern> {
        &self.0
    }

    fn get_pattern_chain_arena(&self) -> &impl Arena<Chain<Pattern>> {
        &self.1
    }

    fn get_timed_step_arena(&self) -> &impl Arena<TimedStep> {
        &self.2
    }

    fn get_timed_step_chain_arena(&self) -> &impl Arena<Chain<TimedStep>> {
        &self.3
    }
}
