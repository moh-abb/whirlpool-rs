use alloc::rc::Rc;
use core::array;
use core::fmt::Debug;

use proptest::strategy::Strategy;
use proptest::test_runner::TestRunner;

use crate::mem::SharedArena;
use crate::mem::SharedArenaRef;

/// Traits representing functions with static lifetimes, that take a mutable
/// reference to `Arenas` and produce an [ArenaResult].
/// This can be thought of as an impure generator function which can modify the
/// arena.
pub trait ArenasToFn<'r, Arenas, T, E>
where
    Self: Fn(SharedArenaRef<'r, Arenas>) -> Result<T, E> + 'r,
    Arenas: 'r,
{
}
impl<'r, Arenas, T, E, F> ArenasToFn<'r, Arenas, T, E> for F
where
    Self: Fn(SharedArenaRef<'r, Arenas>) -> Result<T, E> + 'r,
    Arenas: 'r,
{
}

/// A helper struct to avoid rewriting casting to [ArenasToFn].
pub struct ArenasTo<'r, Arenas, T, E>(
    Rc<dyn ArenasToFn<'r, Arenas, T, E> + 'r>,
);

impl<'r, Arenas, T, E> Debug for ArenasTo<'r, Arenas, T, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ArenasTo")
    }
}

impl<'r, Arenas, T, E> Clone for ArenasTo<'r, Arenas, T, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'r, Arenas, T, E> ArenasTo<'r, Arenas, T, E> {
    pub fn new(f: impl ArenasToFn<'r, Arenas, T, E>) -> Self {
        Self(Rc::new(f) as Rc<dyn ArenasToFn<'r, Arenas, T, E>>)
    }

    pub fn call(&self, arenas: SharedArenaRef<'r, Arenas>) -> Result<T, E> {
        (self.0)(arenas)
    }
}

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
