use proptest::test_runner::TestRunner;

use crate::arena::Arena;
use crate::arena::arena_impl::growable_arena::GrowableArena;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::test::pattern::arbitrary::ArenasTo;
use crate::test::pattern::arbitrary::arb_pattern;

#[derive(Debug)]
pub struct GrowableArenas(
    pub GrowableArena<Pattern>,
    pub GrowableArena<Chain<Pattern>>,
    pub GrowableArena<TimedStep>,
    pub GrowableArena<Chain<TimedStep>>,
);

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

pub fn with_growable_arena_tuple<U>(f: impl FnOnce(&GrowableArenas) -> U) -> U {
    f(&new_growable_arena_tuple())
}

pub fn with_regenerated_arenas(
    f: impl Fn(&GrowableArenas, Index<Pattern>) + Clone,
) {
    let mut test_runner = TestRunner::deterministic();
    let strat = arb_pattern();
    let run_with_growable_arena =
        |pat2: ArenasTo<_, _>, arenas: &GrowableArenas| {
            (f.clone())(arenas, (pat2.clone().0)(arenas).unwrap())
        };
    test_runner
        .run(&strat, move |pat| {
            let pat2 = pat.clone();
            with_growable_arena_tuple(|arenas| {
                run_with_growable_arena(pat2, arenas)
            });
            Ok(())
        })
        .unwrap()
}

pub fn with_reused_arenas(f: impl Fn(&GrowableArenas, Index<Pattern>)) {
    let mut test_runner = TestRunner::deterministic();
    let strat = arb_pattern();
    with_growable_arena_tuple(|arenas| {
        let run_with_growable_arena = |pat: ArenasTo<_, _>| {
            f(arenas, (pat.0)(arenas).unwrap());
            Ok(())
        };
        test_runner
            .run(&strat, run_with_growable_arena)
            .unwrap()
    })
}

pub type TesterFn = fn(&GrowableArenas, Index<Pattern>);
