use core::num::NonZeroU8;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::arena::Arena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::alloc_pattern;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::MultiplePatternDropAdapter;
use crate::ast::pattern::drop::MultipleTimedStepDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::drop::TimedStepDropAdapter;
use crate::ast::pattern::drop::multiple_cons;
use crate::ast::time::CycleTime;
use crate::structures::index::INVALID_INDEX_VALUE;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::silence::arb_pattern_leaf;
use crate::test::pattern::arbitrary::time::arb_positive_cycle_time;

fn pattern_to_timed_step<Arenas: PatternArenas + 'static>(
    x: ArenasTo<Arenas, Index<Pattern>>,
    time_unit: CycleTime,
) -> ArenasTo<Arenas, Index<TimedStep>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let timed_step_arena = arenas.get_timed_step_arena();
        let alloc_pattern = x.call(arenas)?;
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
    time_unit: CycleTime,
    f: fn(Multiple<TimedStep>) -> Pattern,
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
                    let pattern_index = x.call(arenas)?;
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
            f,
            |pattern| match pattern {
                Pattern::TimeCat(multiple) | Pattern::Arrange(multiple) => {
                    multiple
                }
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
                let pattern_index = x.call(arenas)?;
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

fn arb_pattern<Arenas: PatternArenas + 'static>(
    depth: u32,
    max_number_of_nodes: u32,
    items_per_collection: u32,
) -> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    arb_pattern_leaf().prop_recursive(
        depth,
        max_number_of_nodes,
        items_per_collection,
        |inner| {
            let pattern_functions = prop_oneof![
                Just(Pattern::Cat as fn(_) -> _),
                Just(Pattern::Seq as fn(_) -> _),
                Just(Pattern::Stack as fn(_) -> _),
            ];
            let timed_step_functions = prop_oneof![
                Just(Pattern::TimeCat as fn(_) -> _),
                Just(Pattern::Arrange as fn(_) -> _),
            ];
            let opt_time_units = prop_oneof![
                arb_positive_cycle_time::<NonZeroU8>().prop_map(Some),
                Just(None)
            ];
            (
                prop::collection::vec(inner, 1..10),
                opt_time_units,
                pattern_functions,
                timed_step_functions,
            )
                .prop_map(
                    |(xs, opt_time_unit, pattern_func, timed_step_func)| {
                        if let Some(time_unit) = opt_time_unit {
                            pattern_to_time_cat(
                                xs.clone(),
                                time_unit,
                                timed_step_func,
                            )
                        } else {
                            pattern_to_multiple_pattern(xs, pattern_func)
                        }
                    },
                )
        },
    )
}

pub fn arb_small_pattern<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    arb_pattern(5, 40, 7)
}

pub fn arb_large_pattern<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    arb_pattern(8, 256, 10)
}
