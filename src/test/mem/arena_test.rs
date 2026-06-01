use proptest::strategy::Strategy;
use proptest::test_runner::TestRunner;

use crate::test::mem::arenas_to::ArenasTo;

pub trait ArenaTest<Item, Arenas> {
    fn run(arenas: &Arenas, item: Item);
}

pub trait StrategyWithArena<Item, Arenas> {
    fn item_strategy() -> impl Strategy<Value = ArenasTo<Arenas, Item>>;
}

pub trait ArenaTest2<Item, Arenas> {
    fn run(arenas: &Arenas, item1: Item, item2: Item);
}

pub fn with_regenerated_arenas<
    TestItem: 'static,
    Test: ArenaTest<TestItem, Arenas>,
    Arenas: Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem, Arenas>,
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
    Test: ArenaTest<TestItem, Arenas>,
    Arenas: Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem, Arenas>,
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
    Test: ArenaTest2<TestItem, Arenas>,
    Arenas: Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem, Arenas>,
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
    Test: ArenaTest2<TestItem, Arenas>,
    Arenas: Default + 'static,
    ItemStrategy: StrategyWithArena<TestItem, Arenas>,
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
