use proptest::test_runner::TestRunner;

use crate::arena::arena_impl::growable_arena::GrowableArena;
use crate::arena::chain::Chain;
use crate::arena::equality::ArenaEq;
use crate::arena::handler::ArenaHandler;
use crate::arena::tuple::ArenaTuple;
use crate::arena::tuple::DynArenasOf;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::test::arbitrary::arb_pattern;

fn with_growable_arena_tuple<U>(
    f: impl FnOnce(DynArenasOf<'_, Pattern>) -> U,
) -> U {
    let pattern_arena = GrowableArena::<Pattern>::new();
    let chain_arena = GrowableArena::<Chain<Pattern>>::new();
    let timed_step_arena = GrowableArena::<TimedStep>::new();
    let timed_step_chain_arena = GrowableArena::<Chain<TimedStep>>::new();
    let arena_tuple = (
        pattern_arena,
        (chain_arena, (timed_step_arena, (timed_step_chain_arena, ()))),
    );
    let dyn_arenas = ArenaTuple::to_dyn_arenas(&arena_tuple);
    f(dyn_arenas)
}

fn with_regenerated_arenas(f: impl Fn(DynArenasOf<'_, Pattern>, Pattern)) {
    let mut test_runner = TestRunner::deterministic();
    let strat = arb_pattern();
    test_runner
        .run(&strat, |pat| {
            with_growable_arena_tuple(|arena_tuple| {
                f(arena_tuple, (pat.0)(arena_tuple).unwrap());
            });
            Ok(())
        })
        .unwrap()
}

fn with_reused_arenas(f: impl Fn(DynArenasOf<'_, Pattern>, Pattern)) {
    let mut test_runner = TestRunner::deterministic();
    let strat = arb_pattern();
    with_growable_arena_tuple(|arena_tuple| {
        test_runner
            .run(&strat, |pat| {
                f(arena_tuple, (pat.0)(arena_tuple).unwrap());
                Ok(())
            })
            .unwrap()
    })
}

type TesterFn = fn(DynArenasOf<'_, Pattern>, Pattern);
const DO_NOTHING: TesterFn = |_, _| ();
const DROP_PATTERN: TesterFn =
    |arena_tuple, pattern| pattern.drop_in(&arena_tuple);
const CLONE_AND_CHECK_EQUAL: TesterFn = |arena_tuple, pattern| {
    let cloned = pattern.clone_in(&arena_tuple);
    let equals = ArenaEq::eq_in(&pattern, &cloned, &arena_tuple, &arena_tuple);
    assert!(equals, "Pattern {pattern:?} and cloned {cloned:?} are distinct")
};
const CLONE_AND_DROP_AND_CHECK_EQUAL: TesterFn = |arena_tuple, pattern| {
    let cloned = pattern.clone_in(&arena_tuple);
    let cloned_2 = cloned.clone_in(&arena_tuple);
    cloned.drop_in(&arena_tuple);
    let cloned_3 = cloned_2.clone_in(&arena_tuple);
    cloned_2.drop_in(&arena_tuple);
    let equals =
        ArenaEq::eq_in(&cloned_3, &pattern, &arena_tuple, &arena_tuple);
    assert!(equals, "Pattern {pattern:?} and cloned {cloned_3:?} are distinct")
};

#[test]
fn can_allocate_in_growable_arenas_once() {
    with_regenerated_arenas(DO_NOTHING)
}

#[test]
fn can_allocate_in_growable_arenas_multiple() {
    with_reused_arenas(DO_NOTHING)
}

#[test]
fn can_allocate_then_deallocate_once() {
    with_regenerated_arenas(DROP_PATTERN)
}

#[test]
fn can_allocate_then_deallocate_multiple() {
    with_reused_arenas(DROP_PATTERN)
}

#[test]
fn can_clone_and_result_is_equal_once() {
    with_regenerated_arenas(CLONE_AND_CHECK_EQUAL)
}

#[test]
fn can_clone_and_result_is_equal_multiple() {
    with_reused_arenas(CLONE_AND_CHECK_EQUAL)
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_once() {
    with_regenerated_arenas(CLONE_AND_DROP_AND_CHECK_EQUAL);
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_multiple() {
    with_reused_arenas(CLONE_AND_DROP_AND_CHECK_EQUAL);
}
