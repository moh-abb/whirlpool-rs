use core::num::NonZeroU8;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::drop::MultiplePatternDropAdapter;
use crate::ast::pattern::drop::MultipleTimedStepDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::drop::TimedStepDropAdapter;
use crate::mem::Arena;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::silence::arb_pattern_leaf;
use crate::test::pattern::arbitrary::time::arb_positive_cycle_time;

fn pattern_to_time_cat<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    time_units: Vec<CycleTime>,
    f: fn(Multiple<TimedStep>) -> Pattern,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let pattern_arena = arenas.get_pattern_arena();
        let timed_step_arena = arenas.get_timed_step_arena();
        let chain_arena = arenas.get_timed_step_chain_arena();
        let mut multiple_adapter = xs
            .iter()
            .cloned()
            .zip(&time_units)
            .try_fold(
                MultipleTimedStepDropAdapter(
                    Some(Multiple::new_empty()),
                    arenas,
                ),
                |mut multiple_adapter, (x, duration)| {
                    // Obtain the arbitrary pattern which will form
                    // the TimedStep.
                    let pattern_index = x.call(arenas)?;

                    // Allocate the TimedStep.
                    let mut pattern_adapter =
                        PatternDropAdapter(Some(pattern_index.clone()), arenas);
                    let timed_step_index = timed_step_arena
                        .alloc(TimedStep(*duration, pattern_index))?;
                    pattern_adapter.0.take();

                    // Append to the Multiple<TimedStep>.
                    let mut timed_step_adapter = TimedStepDropAdapter(
                        Some(timed_step_index.clone()),
                        arenas,
                    );
                    let timed_step_chain = Chain(timed_step_index, None, None);
                    let chain_index = chain_arena.alloc(timed_step_chain)?;
                    timed_step_adapter.0.take();

                    let multiple = multiple_adapter.0.as_mut().unwrap();
                    multiple.push_back(
                        arenas.get_timed_step_chain_arena(),
                        chain_index,
                    );
                    Ok(multiple_adapter)
                },
            )?;
        let multiple = multiple_adapter.0.clone().unwrap();
        let result = pattern_arena.alloc(f(multiple))?;
        multiple_adapter.0.take();
        Ok(result)
    })
}

fn pattern_to_multiple_pattern<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<Pattern>>>,
    f: fn(Multiple<Pattern>) -> Pattern,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let pattern_arena = arenas.get_pattern_arena();
        let chain_arena = arenas.get_pattern_chain_arena();
        let mut multiple_adapter = xs.iter().cloned().try_fold(
            MultiplePatternDropAdapter(Some(Multiple::new_empty()), arenas),
            |mut multiple_adapter, x| {
                // Obtain the arbitrary pattern.
                let pattern_index = x.call(arenas)?;
                let mut item_adapter =
                    PatternDropAdapter(Some(pattern_index.clone()), arenas);

                // Append to the Multiple<Pattern>.
                let chain_index =
                    chain_arena.alloc(Chain(pattern_index, None, None))?;
                item_adapter.0.take();

                let multiple = multiple_adapter.0.as_mut().unwrap();
                multiple
                    .push_back(arenas.get_pattern_chain_arena(), chain_index);
                Ok(multiple_adapter)
            },
        )?;
        let multiple = multiple_adapter.0.clone().unwrap();
        let result = pattern_arena.alloc(f(multiple))?;
        multiple_adapter.0.take();
        Ok(result)
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
            let time_units = prop::collection::vec(
                arb_positive_cycle_time::<NonZeroU8>(),
                1..10,
            );
            let opt_time_units =
                prop_oneof![time_units.prop_map(Some), Just(None)];
            (
                prop::collection::vec(inner, 1..10),
                opt_time_units,
                pattern_functions,
                timed_step_functions,
            )
                .prop_map(
                    |(xs, opt_time_units, pattern_func, timed_step_func)| {
                        if let Some(time_units) =
                            opt_time_units.filter(|_| false)
                        {
                            pattern_to_time_cat(
                                xs.clone(),
                                time_units,
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
