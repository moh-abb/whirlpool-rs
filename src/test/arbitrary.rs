use core::fmt::Debug;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::alloc_types::Rc;
use crate::arena::chain::Chain;
use crate::arena::error::ArenaResult;
use crate::arena::handler::ArenaHandler;
use crate::ast::note::NoteUnit;
use crate::ast::pattern::Pattern;

/// Traits representing functions with static lifetimes, that take a tuple of
/// `dyn Arena<_>` and produce an [ArenaResult].
/// This can be thought of as an impure generator function which can modify the
/// arena.
trait ArenasToFn<T: ArenaHandler>
where
    Self: Fn(<T as ArenaHandler>::DynArenas<'_>) -> ArenaResult<T>,
    Self: 'static,
{
}
impl<T: ArenaHandler, F> ArenasToFn<T> for F
where
    Self: Fn(<T as ArenaHandler>::DynArenas<'_>) -> ArenaResult<T>,
    Self: 'static,
{
}

/// A helper struct to avoid rewriting casting to [ArenasToFn].
struct ArenasTo<T: ArenaHandler>(Rc<dyn ArenasToFn<T>>);

impl<T: ArenaHandler> Debug for ArenasTo<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ArenasTo")
    }
}

impl<T: ArenaHandler> Clone for ArenasTo<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ArenaHandler> ArenasTo<T> {
    fn new(f: impl ArenasToFn<T>) -> Self {
        Self(Rc::new(f) as Rc<dyn ArenasToFn<T>>)
    }
}

type DynArenasOf<'a, T> = <T as ArenaHandler>::DynArenas<'a>;

#[allow(unused)]
fn arb_pattern() -> impl Strategy<Value = ArenasTo<Pattern>> {
    let leaf = prop_oneof![
        Just(ArenasTo::new(|_| Ok(Pattern::Silence))),
        any::<NoteUnit>()
            .prop_map(|n| ArenasTo::new(move |_| Ok(Pattern::Note(n)))),
    ];
    leaf.prop_recursive(
        8,   // levels deep
        256, // maximum number of nodes
        10,  // up to 10 items per collection
        |inner| {
            let functions = prop_oneof![
                Just(Pattern::Cat as fn(_) -> _),
                Just(Pattern::Seq as fn(_) -> _),
                Just(Pattern::Stack as fn(_) -> _),
            ];
            (prop::collection::vec(inner, 0..10), functions).prop_map(
                |(xs, f)| {
                    ArenasTo::new(move |arenas: DynArenasOf<'_, Pattern>| {
                        let (
                            pattern_arena,
                            (
                                chain_arena,
                                (
                                    _timed_step_arena,
                                    (_timed_step_chain_arena, ()),
                                ),
                            ),
                        ) = arenas;
                        let chain_cons = |chain, x: ArenasTo<Pattern>| {
                            ArenaResult::Ok(Chain::Cons {
                                head: pattern_arena.alloc((x.0)(arenas)?)?,
                                tail: chain_arena.alloc(chain)?,
                            })
                        };
                        xs.iter()
                            .cloned()
                            .try_fold(Chain::Nil, chain_cons)
                            .map(f)
                    })
                },
            )
        },
    )
}

mod tests {
    use proptest::prelude::TestCaseError;
    use proptest::test_runner::TestRunner;

    use crate::arena::arena_impl::growable_arena::GrowableArena;
    use crate::arena::chain::Chain;
    use crate::arena::error::ArenaError;
    use crate::arena::tuple::ArenaTuple;
    use crate::ast::pattern::Pattern;
    use crate::ast::pattern::TimedStep;

    #[test]
    fn allocation_in_growable_arenas_should_not_fail() {
        let mut test_runner = TestRunner::deterministic();
        let strat = super::arb_pattern();
        let test_run_result = test_runner.run(&strat, |pat| {
            let pattern_arena = GrowableArena::<Pattern>::new();
            let chain_arena = GrowableArena::<Chain<Pattern>>::new();
            let timed_step_arena = GrowableArena::<TimedStep>::new();
            let timed_step_chain_arena =
                GrowableArena::<Chain<TimedStep>>::new();
            let arena_tuple = (
                pattern_arena,
                (chain_arena, (timed_step_arena, (timed_step_chain_arena, ()))),
            );
            let generated_pattern = (pat.0)(arena_tuple.to_dyn_arenas());
            let error_message = |e: ArenaError| {
                format!(
                    "Generating pattern with arena tuple {arena_tuple:?}
                 failed and gave {e:?}"
                )
            };
            generated_pattern
                .map(|_| ())
                .map_err(|e| TestCaseError::fail(error_message(e)))
        });
        test_run_result.unwrap()
    }
}
