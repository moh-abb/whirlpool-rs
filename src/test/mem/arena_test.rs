use core::cell::RefCell;
use core::ops::DerefMut;

use proptest::strategy::Strategy;
use proptest::test_runner::TestRunner;

use crate::test::mem::arenas_to::ArenasTo;

pub trait ArenaTest<Item, Arenas> {
    fn run(arenas: &mut Arenas, item: Item);
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
    let run_with_arena = |pat2: ArenasTo<_, _>, arenas: &mut Arenas| {
        let item = pat2.call(arenas).unwrap();
        Test::run(arenas, item)
    };
    test_runner
        .run(&strat, move |pat| {
            let pat2 = pat.clone();
            let mut arenas = Arenas::default();
            run_with_arena(pat2, &mut arenas);
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
    let arenas = RefCell::new(Arenas::default());
    let run_with_arena = |pat: ArenasTo<_, _>| {
        let mut borrowed_arenas = arenas.borrow_mut();
        let item = pat
            .call(borrowed_arenas.deref_mut())
            .unwrap();
        Test::run(borrowed_arenas.deref_mut(), item);
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
            let mut arenas = Arenas::default();
            let (x, y) = pat.call(&mut arenas).unwrap();
            Test::run(&mut arenas, x, y);
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
    let arenas = RefCell::new(Arenas::default());
    let run_with_arena = |pat: ArenasTo<_, _>| {
        let mut borrowed_arenas = arenas.borrow_mut();
        let (x, y) = pat
            .call(borrowed_arenas.deref_mut())
            .unwrap();
        Test::run(borrowed_arenas.deref_mut(), x, y);
        Ok(())
    };
    test_runner
        .run(&strat, run_with_arena)
        .unwrap()
}
