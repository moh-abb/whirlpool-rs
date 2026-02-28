use proptest::prelude::Strategy;
use proptest::test_runner::TestRunner;

use crate::arena::Arena;
use crate::arena::arena_impl::growable_arena::GrowableArena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::structures::chain::Chain;
use crate::structures::index::Index;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::pattern::arb_pattern;

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

pub fn with_regenerated_arenas<
    TestItem: 'static,
    Test: ArenaTest<TestItem>,
    Arenas: PatternArenas + Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem>,
>() {
    let mut test_runner = TestRunner::deterministic();
    let strat = ItemStrategy::item_strategy();
    let run_with_arena = |pat2: ArenasTo<_, _>, arenas: &Arenas| {
        Test::run(arenas, pat2.call(arenas).unwrap())
    };
    test_runner
        .run(&strat, move |pat| {
            let pat2 = pat.clone();
            let arenas = Arenas::default();
            run_with_arena(pat2, &arenas);
            Ok(())
        })
        .unwrap()
}

pub fn with_reused_arenas<
    TestItem: 'static,
    Test: ArenaTest<TestItem>,
    Arenas: PatternArenas + Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem>,
>() {
    let mut test_runner = TestRunner::deterministic();
    let strat = ItemStrategy::item_strategy();
    let arenas = Arenas::default();
    let run_with_arena = |pat: ArenasTo<_, _>| {
        Test::run(&arenas, pat.call(&arenas).unwrap());
        Ok(())
    };
    test_runner
        .run(&strat, run_with_arena)
        .unwrap()
}

pub fn with_regenerated_arenas_double<
    TestItem: 'static,
    Test: ArenaTest2<TestItem>,
    Arenas: PatternArenas + Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem>,
>() {
    let mut test_runner = TestRunner::deterministic();
    let strat = (ItemStrategy::item_strategy(), ItemStrategy::item_strategy())
        .prop_map(|(x, y)| {
            ArenasTo::new(move |arenas| Ok((x.call(arenas)?, y.call(arenas)?)))
        });
    test_runner
        .run(&strat, move |pat| {
            let pat = pat.clone();
            let arenas = Arenas::default();
            let (x, y) = pat.call(&arenas).unwrap();
            Test::run(&arenas, x, y);
            Ok(())
        })
        .unwrap()
}

pub fn with_reused_arenas_double<
    TestItem: 'static,
    Test: ArenaTest2<TestItem>,
    Arenas: PatternArenas + Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem>,
>() {
    let mut test_runner = TestRunner::deterministic();
    let strat = (ItemStrategy::item_strategy(), ItemStrategy::item_strategy())
        .prop_map(|(x, y)| {
            ArenasTo::new(move |arenas| Ok((x.call(arenas)?, y.call(arenas)?)))
        });
    let arenas = Arenas::default();
    let run_with_arena = |pat: ArenasTo<_, _>| {
        let (x, y) = pat.call(&arenas).unwrap();
        Test::run(&arenas, x, y);
        Ok(())
    };
    test_runner
        .run(&strat, run_with_arena)
        .unwrap()
}

pub trait ArenaTest<Item> {
    fn run(arenas: &impl PatternArenas, item: Item);
}

pub trait StrategyWithArena<Item> {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, Item>>;
}

pub struct AnyPatternStrategy;
impl StrategyWithArena<Index<Pattern>> for AnyPatternStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
        arb_pattern()
    }
}

pub trait ArenaTest2<Item> {
    fn run(arenas: &impl PatternArenas, item1: Item, item2: Item);
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
