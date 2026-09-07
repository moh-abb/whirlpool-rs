use core::array;
use core::fmt::Debug;

use proptest::strategy::Strategy;
use proptest::test_runner::TestRunner;

use crate::mem::arena::arena_impl::shared_arena::SharedArena;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;
use crate::test::mem::arenas_to::ArenasTo;

pub trait ArenaTest<Item, Arenas> {
    fn run<'r>(shared_arena_ref: SharedArenaRef<'r, Arenas>, item: Item);
}

pub trait StrategyWithArena<Item, Arenas, Error> {
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, Item, Error>>
    where
        Arenas: 'r;
}

pub trait ArenaTest2<Item, Arenas> {
    fn run<'r>(arenas: SharedArenaRef<'r, Arenas>, item1: Item, item2: Item);
}

pub fn arenas_test_run<
    TestItem,
    Test: ArenaTest<TestItem, Arenas>,
    Arenas: Default,
    ItemStrategy: StrategyWithArena<TestItem, Arenas, Error>,
    Error: Debug,
>() {
    let mut test_runner = TestRunner::deterministic();
    let shared_arena = SharedArena::new(Arenas::default());
    let [ref_1, ref_2] = array::repeat(shared_arena.make_ref());
    let strat = ItemStrategy::item_strategy();
    test_runner
        .run(&strat, move |pat| {
            let item = pat.call(ref_1.clone()).unwrap();
            Test::run(ref_2.clone(), item);
            Ok(())
        })
        .unwrap()
}

pub fn double_arenas_test_run<
    TestItem,
    Test: ArenaTest2<TestItem, Arenas>,
    Arenas: Default,
    ItemStrategy: StrategyWithArena<TestItem, Arenas, Error>,
    Error: Debug + 'static,
>() {
    let mut test_runner = TestRunner::deterministic();
    let shared_arena = SharedArena::new(Arenas::default());
    let [ref_1, ref_2] = array::repeat(shared_arena.make_ref());
    let strat = (ItemStrategy::item_strategy(), ItemStrategy::item_strategy())
        .prop_map(|(x, y)| {
            ArenasTo::new(move |arenas| {
                Result::<_, Error>::Ok((
                    x.call(arenas.clone())?,
                    y.call(arenas)?,
                ))
            })
        });
    test_runner
        .run(&strat, |pat: ArenasTo<_, _, _>| {
            let (x, y) = pat.call(ref_1.clone()).unwrap();
            Test::run(ref_2.clone(), x, y);
            Ok(())
        })
        .unwrap()
}
