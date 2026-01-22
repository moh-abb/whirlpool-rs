use core::fmt::Debug;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::alloc_types::Rc;
use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::error::ArenaResult;
use crate::arena::index::INVALID_INDEX_VALUE;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimeUnit;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::make_chain_output_cons;
use crate::ast::pattern::clone::make_chain_output_nil;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::PatternChainDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::drop::TimedStepChainDropAdapter;
use crate::ast::pattern::drop::TimedStepDropAdapter;
use crate::ast::pattern::note::NoteUnit;

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

type ResultWithLength<D> = (u16, ArenaResult<D>);

fn chain_cons<
    'a,
    Item: ArenaItem,
    Arenas: PatternArenas,
    Adapter: DropAdapter<'a, Item, Arenas>,
    ChainAdapter: DropAdapter<'a, Chain<Item>, Arenas>,
>(
    arenas: &'a Arenas,
    acc: ResultWithLength<ChainAdapter>,
    x: ArenaResult<Adapter>,
    chain_arena: &'a impl Arena<Chain<Item>>,
) -> ResultWithLength<ChainAdapter> {
    let (length, chain_drop_adapter) = acc;
    let cons_chain_drop_adapter =
        make_chain_output_cons(arenas, chain_arena, chain_drop_adapter, || x);
    (length + 1, cons_chain_drop_adapter)
}

trait ItemAndChainAdapter<Item: ArenaItem, Arenas: PatternArenas>: 'static {
    type ItemAdapter<'a>: DropAdapter<'a, Item, Arenas>
    where
        Arenas: 'a;
    type ChainAdapter<'a>: DropAdapter<'a, Chain<Item>, Arenas>
    where
        Arenas: 'a;
}

struct PatternAndChainAdapter;
impl<Arenas: PatternArenas> ItemAndChainAdapter<Pattern, Arenas>
    for PatternAndChainAdapter
{
    type ItemAdapter<'a>
        = PatternDropAdapter<'a, Arenas>
    where
        Arenas: 'a;
    type ChainAdapter<'a>
        = PatternChainDropAdapter<'a, Arenas>
    where
        Arenas: 'a;
}

struct TimedStepAndChainAdapter;
impl<Arenas: PatternArenas> ItemAndChainAdapter<TimedStep, Arenas>
    for TimedStepAndChainAdapter
{
    type ItemAdapter<'a>
        = TimedStepDropAdapter<'a, Arenas>
    where
        Arenas: 'a;
    type ChainAdapter<'a>
        = TimedStepChainDropAdapter<'a, Arenas>
    where
        Arenas: 'a;
}

#[allow(type_alias_bounds)]
type ChainConsFn<Item, Arenas, Adapters: ItemAndChainAdapter<Item, Arenas>> =
    for<'a> fn(
        &'a Arenas,
        ResultWithLength<Adapters::ChainAdapter<'a>>,
        ArenaResult<Adapters::ItemAdapter<'a>>,
    ) -> ResultWithLength<Adapters::ChainAdapter<'a>>;

fn build_from_chain<
    Item: ArenaItem,
    Arenas: PatternArenas + 'static,
    Adapters: ItemAndChainAdapter<Item, Arenas>,
>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    make_drop_adapter: impl for<'a> Fn(
        Index<Pattern>,
        &'a Arenas,
    )
        -> ArenaResult<Adapters::ItemAdapter<'a>>
    + 'static,
    chain_cons: ChainConsFn<Item, Arenas, Adapters>,
    make_empty_chain: fn(&Arenas) -> ArenaResult<Adapters::ChainAdapter<'_>>,
    make_item: impl for<'a> Fn(
        &'a Arenas,
        u16,
        Adapters::ChainAdapter<'a>,
    ) -> ArenaResult<Index<Pattern>>
    + 'static,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let to_drop_adapter =
            |x: ArenaResult<Index<Pattern>>| make_drop_adapter(x?, arenas);

        let pattern_chain_initial = (0, make_empty_chain(arenas));

        // Convert to drop adapters, so that all if allocation fails halfway
        // through and the `Vec` is dropped, then all the subpatterns are freed.
        let xs_drop_adapters = xs
            .iter()
            .cloned()
            .map(|x| (x.0)(arenas))
            .map(to_drop_adapter)
            .collect::<Vec<ArenaResult<_>>>();

        let fold_result: ResultWithLength<_> = xs_drop_adapters
            .into_iter()
            .fold(pattern_chain_initial, |acc, x| chain_cons(arenas, acc, x));
        let (fold_length, drop_adapter_result) = fold_result;
        make_item(arenas, fold_length, drop_adapter_result?)
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

fn pattern_leaf<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    prop_oneof![
        Just(silence_leaf()),
        any::<NoteUnit>().prop_map(note_unit_leaf)
    ]
}

fn make_timed_step_drop_adapter<Arenas: PatternArenas>(
    time_unit: TimeUnit,
    index: Index<Pattern>,
    arenas: &Arenas,
) -> ArenaResult<TimedStepDropAdapter<'_, Arenas>> {
    // First try to allocate space for the `TimedStep`.
    let invalid_timed_step =
        TimedStep(time_unit, Index::new(INVALID_INDEX_VALUE));
    let mut pattern_drop_adapter = PatternDropAdapter::new(index, arenas);
    let timed_step_arena = arenas.get_timed_step_arena();
    let timed_step_index = timed_step_arena.alloc(invalid_timed_step)?;
    let fill_timed_step_index = |timed_step: &mut TimedStep| {
        timed_step.1 = pattern_drop_adapter.take_index()
    };
    timed_step_arena
        .inspect_mut(timed_step_index.clone(), fill_timed_step_index)
        .unwrap();
    Ok(TimedStepDropAdapter::new(timed_step_index, arenas))
}

fn multiple_item_to_pattern<'a, Item: ArenaItem, Arenas: PatternArenas>(
    arenas: &'a Arenas,
    chain_arena: &'a impl Arena<Chain<Item>>,
    length: u16,
    mut adapter: impl DropAdapter<'a, Chain<Item>, Arenas>,
    make_pattern: impl FnOnce(Multiple<Item>) -> Pattern,
    get_multiple_from_pattern: impl FnOnce(&mut Pattern) -> &mut Multiple<Item>,
) -> ArenaResult<Index<Pattern>> {
    // Attempt to allocate a new `TimeCat`.
    let pattern_arena = arenas.get_pattern_arena();
    let invalid_multiple = Multiple {
        length: INVALID_INDEX_VALUE,
        index: Index::new(INVALID_INDEX_VALUE),
    };
    let pattern_index = pattern_arena.alloc(make_pattern(invalid_multiple))?;
    let alloc_chain_index = adapter.take_index();
    let update_multiple = |multiple: &mut Multiple<_>| {
        let valid_multiple = Multiple { length, index: alloc_chain_index };
        valid_multiple.verify_length(chain_arena);
        let _ = core::mem::replace(multiple, valid_multiple);
    };
    pattern_arena.inspect_mut(pattern_index.clone(), |pattern| {
        update_multiple(get_multiple_from_pattern(pattern))
    })?;
    Ok(pattern_index)
}

fn pattern_to_time_cat<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    time_unit: TimeUnit,
) -> ArenasTo<Arenas, Index<Pattern>> {
    build_from_chain::<_, _, TimedStepAndChainAdapter>(
        xs,
        move |index, arenas: &Arenas| {
            make_timed_step_drop_adapter(time_unit, index, arenas)
        },
        |arenas, result_with_length, adapter| {
            chain_cons(
                arenas,
                result_with_length,
                adapter,
                arenas.get_timed_step_chain_arena(),
            )
        },
        |arenas| {
            make_chain_output_nil(arenas, arenas.get_timed_step_chain_arena())
        },
        |arenas: &Arenas, length, adapter| {
            multiple_item_to_pattern(
                arenas,
                arenas.get_timed_step_chain_arena(),
                length,
                adapter,
                Pattern::TimeCat,
                |pattern| match pattern {
                    Pattern::TimeCat(multiple) => multiple,
                    _ => unreachable!(),
                },
            )
        },
    )
}

fn pattern_to_multiple_pattern<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    f: fn(Multiple<Pattern>) -> Pattern,
) -> ArenasTo<Arenas, Index<Pattern>> {
    build_from_chain::<_, _, PatternAndChainAdapter>(
        xs,
        |index, arenas| Ok(PatternDropAdapter::new(index, arenas)),
        |arenas: &Arenas, result_with_length, adapter| {
            chain_cons(
                arenas,
                result_with_length,
                adapter,
                arenas.get_pattern_chain_arena(),
            )
        },
        |arenas: &Arenas| {
            make_chain_output_nil(arenas, arenas.get_pattern_chain_arena())
        },
        move |arenas: &Arenas, length, adapter| {
            multiple_item_to_pattern(
                arenas,
                arenas.get_pattern_chain_arena(),
                length,
                adapter,
                f,
                |pattern| match pattern {
                    Pattern::Cat(multiple)
                    | Pattern::Seq(multiple)
                    | Pattern::Stack(multiple) => multiple,
                    _ => unreachable!(),
                },
            )
        },
    )
}

pub fn arb_pattern<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    let result = pattern_leaf().prop_recursive(
        8,   // levels deep
        256, // maximum number of nodes
        10,  // up to 10 items per collection
        |inner| {
            let functions = prop_oneof![
                Just(Pattern::Cat as fn(_) -> _),
                Just(Pattern::Seq as fn(_) -> _),
                Just(Pattern::Stack as fn(_) -> _),
            ];
            (
                prop::collection::vec(inner, 0..10),
                functions,
                any::<Option<TimeUnit>>(),
            )
                .prop_map(|(xs, f, opt_time_unit)| {
                    if let Some(time_unit) = opt_time_unit {
                        pattern_to_time_cat(xs, time_unit)
                    } else {
                        pattern_to_multiple_pattern(xs, f)
                    }
                })
        },
    );

    result
}
