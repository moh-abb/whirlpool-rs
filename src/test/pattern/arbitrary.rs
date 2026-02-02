use core::fmt::Debug;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::alloc_types::Rc;
use crate::arena::Arena;
use crate::arena::error::ArenaResult;
use crate::structures::index::INVALID_INDEX_VALUE;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimeUnit;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::alloc_pattern;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::MultiplePatternDropAdapter;
use crate::ast::pattern::drop::MultipleTimedStepDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::drop::TimedStepDropAdapter;
use crate::ast::pattern::drop::multiple_cons;
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
    pub fn new(f: impl ArenasToFn<Arenas, T> + 'static) -> Self {
        Self(Rc::new(f) as Rc<dyn ArenasToFn<Arenas, T>>)
    }
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

fn pattern_to_timed_step<Arenas: PatternArenas + 'static>(
    x: ArenasTo<Arenas, Index<Pattern>>,
    time_unit: TimeUnit,
) -> ArenasTo<Arenas, Index<TimedStep>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let timed_step_arena = arenas.get_timed_step_arena();
        let alloc_pattern = (x.0)(arenas)?;
        let mut pattern_drop_adapter =
            PatternDropAdapter::new(alloc_pattern, arenas);
        // Allocation starts here.
        let timed_step_index = timed_step_arena
            .alloc(TimedStep(time_unit, Index::new(INVALID_INDEX_VALUE)))?;
        // Allocation ends here.
        timed_step_arena
            .inspect_mut(timed_step_index.clone(), |timed_step| {
                let _ = core::mem::replace(
                    &mut timed_step.1,
                    pattern_drop_adapter.take_item(),
                );
            })
            .unwrap();
        Ok(timed_step_index)
    })
}

fn pattern_to_time_cat<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    time_unit: TimeUnit,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let chain_arena = arenas.get_timed_step_chain_arena();
        let multiple_adapter = xs
            .iter()
            .cloned()
            .map(|x| pattern_to_timed_step(x, time_unit))
            .try_fold(
                MultipleTimedStepDropAdapter::new(
                    Multiple::new_empty(),
                    arenas,
                ),
                |multiple_adapter, x| {
                    let pattern_index = (x.0)(arenas)?;
                    let item_adapter =
                        TimedStepDropAdapter::new(pattern_index, arenas);
                    multiple_cons(
                        multiple_adapter,
                        item_adapter,
                        arenas,
                        chain_arena,
                    )
                },
            )?;
        let mut drop_adapter = alloc_pattern(
            arenas,
            Pattern::TimeCat,
            |pattern| match pattern {
                Pattern::TimeCat(multiple) => multiple,
                _ => unreachable!(),
            },
            multiple_adapter,
        )?;
        Ok(drop_adapter.take_item())
    })
}

fn pattern_to_multiple_pattern<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    f: fn(Multiple<Pattern>) -> Pattern,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let chain_arena = arenas.get_pattern_chain_arena();
        let multiple_adapter = xs.iter().cloned().try_fold(
            MultiplePatternDropAdapter::new(Multiple::new_empty(), arenas),
            |multiple_adapter, x| {
                let pattern_index = (x.0)(arenas)?;
                let item_adapter =
                    PatternDropAdapter::new(pattern_index, arenas);
                multiple_cons(
                    multiple_adapter,
                    item_adapter,
                    arenas,
                    chain_arena,
                )
            },
        )?;
        let mut drop_adapter = alloc_pattern(
            arenas,
            f,
            |pattern| match pattern {
                Pattern::Cat(multiple)
                | Pattern::Seq(multiple)
                | Pattern::Stack(multiple) => multiple,
                _ => unreachable!(),
            },
            multiple_adapter,
        )?;
        Ok(drop_adapter.take_item())
    })
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
