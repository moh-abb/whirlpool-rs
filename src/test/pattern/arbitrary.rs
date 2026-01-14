use core::fmt::Debug;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::alloc_types::Rc;
use crate::arena::Arena;
use crate::arena::chain::Chain;
use crate::arena::error::ArenaResult;
use crate::arena::extension::Inspect;
use crate::arena::index::INVALID_INDEX_VALUE;
use crate::arena::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::clone::make_chain_output_cons;
use crate::ast::pattern::clone::make_chain_output_nil;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::PatternChainDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::arenas::PatternArenas;

/// Traits representing functions with static lifetimes, that take a tuple of
/// `dyn Arena<_>` and produce an [ArenaResult].
/// This can be thought of as an impure generator function which can modify the
/// arena.
pub trait ArenasToFn<Arenas, T>
where
    Self: Fn(&Arenas) -> ArenaResult<T>,
    Arenas: PatternArenas,
{
}
impl<Arenas, T, F> ArenasToFn<Arenas, T> for F
where
    Self: Fn(&Arenas) -> ArenaResult<T>,
    Arenas: PatternArenas,
{
}

/// A helper struct to avoid rewriting casting to [ArenasToFn].
pub struct ArenasTo<Arenas, T>(pub Rc<dyn ArenasToFn<Arenas, T>>);

impl<Arenas, T> Debug for ArenasTo<Arenas, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ArenasTo")
    }
}

impl<Arenas, T> Clone for ArenasTo<Arenas, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Arenas: PatternArenas, T> ArenasTo<Arenas, T> {
    fn new(f: impl ArenasToFn<Arenas, T> + 'static) -> Self {
        Self(Rc::new(f) as Rc<dyn ArenasToFn<Arenas, T>>)
    }
}

fn pattern_chain_cons<'a, Arenas: PatternArenas>(
    arenas: &'a Arenas,
    acc: ArenaResult<PatternChainDropAdapter<'a, Arenas>>,
    x: ArenaResult<PatternDropAdapter<'a, Arenas>>,
) -> ArenaResult<PatternChainDropAdapter<'a, Arenas>> {
    make_chain_output_cons(arenas, arenas.get_pattern_chain_arena(), acc, || x)
}

fn build_from_chain<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    f: fn(Index<Chain<Pattern>>) -> Pattern,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let to_drop_adapter = |x: ArenaResult<Index<Pattern>>| {
            Ok(PatternDropAdapter::new(x?, arenas))
        };

        let pattern_chain_nil =
            make_chain_output_nil(arenas, arenas.get_pattern_chain_arena());

        let make_pattern = move |mut res: PatternChainDropAdapter<'_, _>| {
            // Attempt to allocate a new `Pattern`.
            let pattern_arena = arenas.get_pattern_arena();
            let pattern_index =
                pattern_arena.alloc(f(Index::new(INVALID_INDEX_VALUE)))?;
            let alloc_chain_index = res.take_index();
            pattern_arena.inspect_mut(pattern_index.clone(), |pattern| {
                match pattern {
                    Pattern::Cat(chain_index)
                    | Pattern::Seq(chain_index)
                    | Pattern::Stack(chain_index) => {
                        let _ =
                            core::mem::replace(chain_index, alloc_chain_index);
                    }
                    Pattern::TimeCat(_)
                    | Pattern::Note(_)
                    | Pattern::Silence => unreachable!(),
                }
            })?;
            Ok(pattern_index)
        };

        // Convert to drop adapters, so that all if allocation fails halfway
        // through and the `Vec` is dropped, then all the subpatterns are freed.
        let xs_drop_adapters = xs
            .iter()
            .cloned()
            .map(|x| (x.0)(arenas))
            .map(to_drop_adapter)
            .collect::<Vec<ArenaResult<_>>>();

        let fold_result: ArenaResult<PatternChainDropAdapter<'_, Arenas>> =
            xs_drop_adapters
                .into_iter()
                .fold(pattern_chain_nil, |acc, x| {
                    pattern_chain_cons(arenas, acc, x)
                });
        make_pattern(fold_result?)
    })
}

fn silence_leaf<Arenas: PatternArenas>() -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(|arenas: &Arenas| {
        arenas
            .get_pattern_arena()
            .alloc(Pattern::Silence)
    })
}

fn note_unit_leaf<Arenas: PatternArenas>(
    unit: NoteUnit,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        arenas
            .get_pattern_arena()
            .alloc(Pattern::Note(unit))
    })
}

#[allow(unused)]
pub fn arb_pattern<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    let leaf = prop_oneof![
        Just(silence_leaf()),
        any::<NoteUnit>().prop_map(note_unit_leaf)
    ];

    let result = leaf.clone().prop_recursive(
        8,   // levels deep
        256, // maximum number of nodes
        10,  // up to 10 items per collection
        |inner| {
            let functions = prop_oneof![
                Just(Pattern::Cat as fn(_) -> _),
                Just(Pattern::Seq as fn(_) -> _),
                Just(Pattern::Stack as fn(_) -> _),
            ];
            (prop::collection::vec(inner, 0..10), functions)
                .prop_map(|(xs, f)| build_from_chain(xs, f))
        },
    );

    result
}
