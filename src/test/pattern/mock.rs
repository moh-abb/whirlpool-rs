use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::arena::arena_impl::mock_arena::ArenaMocker;
use crate::structures::chain::Chain;

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
