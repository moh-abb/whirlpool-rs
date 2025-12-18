use core::error::Error;

use proptest::prelude::TestCaseError;
use proptest::test_runner::TestRunner;

use crate::arena::arena_impl::growable_arena::GrowableArena;
use crate::arena::chain::Chain;
use crate::arena::error::ArenaError;
use crate::arena::tuple::ArenaTuple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::test::arbitrary::DynArenasOf;
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

#[test]
fn can_allocate_multiple_in_growable_arenas() -> Result<(), impl Error> {
    let mut test_runner = TestRunner::deterministic();
    let strat = arb_pattern();
    with_growable_arena_tuple(|arena_tuple| {
        test_runner.run(&strat, |pat| {
            let generated_pattern = (pat.0)(arena_tuple);
            let error_message = |e: ArenaError| {
                format!(
                    "Generating pattern with arena tuple failed and gave {e:?}"
                )
            };
            generated_pattern
                .map(|_| ())
                .map_err(|e| TestCaseError::fail(error_message(e)))
        })
    })
}
