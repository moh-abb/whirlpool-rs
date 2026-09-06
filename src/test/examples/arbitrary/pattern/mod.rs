use core::num::NonZeroU8;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::PatternArenasExt;
use crate::ast::time::arbitrary::arb_positive_cycle_time;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::arena::arena_impl::shared_arena::SharedArena;
use crate::test::examples::arbitrary::pattern::leaf::arb_pattern_leaf;
use crate::test::mem::arenas_to::ArenasTo;
use crate::test::mem::linked::check_acyclic;

mod leaf;

fn pattern_to_time_cat_or_arrange<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<PatternNode>>>,
    time_units: Vec<CycleTime>,
    f: fn(CycleTime, Multiple<PatternNode>) -> Pattern,
) -> ArenasTo<Arenas, Index<PatternNode>> {
    let total_cycle_time = time_units
        .iter()
        .cloned()
        .sum::<Result<CycleTime, _>>()
        .expect("expected times to not overflow");

    ArenasTo::new(move |arenas: &mut Arenas| {
        let shared_arena = SharedArena::new(arenas);
        let mut result_adapter = shared_arena
            .make_ref()
            .push_dropping(f(total_cycle_time, Multiple::new()))?;
        let result_index = result_adapter.clone_index().unwrap();

        let add_single_timed_step =
            |x: ArenasTo<Arenas, _>, duration: &CycleTime| {
                // Add the TimedStep to the result pattern.
                // This is currently invalid as it has no child pattern.
                let mut shared_ref_1 = shared_arena.make_ref();
                let mut shared_ref_2 = shared_arena.make_ref();
                let timed_step_index = Multiple::push_back(
                    &mut shared_ref_1,
                    &mut shared_ref_2,
                    result_index.clone(),
                    Cow::Owned(PatternNode::new(Pattern::TimedStep(
                        TimedStep(*duration, Multiple::new()),
                    ))),
                )?;

                // Obtain the arbitrary pattern which will form
                // the TimedStep.
                let pattern_index =
                    shared_arena.with_inner_mut(|arenas| x.call(arenas))??;
                // Add the child pattern to the TimedStep.
                Multiple::push_back(
                    &mut shared_ref_1,
                    &mut shared_ref_2,
                    timed_step_index.clone(),
                    Cow::Indexed(pattern_index),
                )?;

                Ok(())
            };

        xs.iter()
            .cloned()
            .zip(&time_units)
            .try_for_each(|(x, duration)| add_single_timed_step(x, duration))?;

        // Now take the result to stop it being dropped.
        result_adapter.take_index();
        Ok(result_index)
    })
}

fn pattern_to_multiple_pattern<Arenas: PatternArenas + 'static>(
    xs: Vec<ArenasTo<Arenas, Index<PatternNode>>>,
    f: fn(Multiple<PatternNode>) -> Pattern,
) -> ArenasTo<Arenas, Index<PatternNode>> {
    ArenasTo::new(move |arenas: &mut Arenas| {
        let shared_arena = SharedArena::new(&mut *arenas);
        let mut result_adapter = shared_arena
            .make_ref()
            .push_dropping(f(Multiple::new()))?;
        let result_index = result_adapter.clone_index().unwrap();

        let add_single_pattern = |x: ArenasTo<Arenas, _>| {
            // Obtain the arbitrary pattern which will form
            // the TimedStep.
            let pattern_index =
                shared_arena.with_inner_mut(|arenas| x.call(arenas))??;

            // Add the child pattern to the result pattern.
            let mut shared_ref_1 = shared_arena.make_ref();
            let mut shared_ref_2 = shared_arena.make_ref();
            Multiple::push_back(
                &mut shared_ref_1,
                &mut shared_ref_2,
                result_index.clone(),
                Cow::Indexed(pattern_index),
            )?;

            Ok(())
        };

        xs.iter()
            .cloned()
            .try_for_each(add_single_pattern)?;

        // Now take the result to stop it being dropped.
        result_adapter.take_index();
        drop(result_adapter);

        check_acyclic(result_index.clone(), arenas);

        Ok(result_index)
    })
}

fn arb_pattern<Arenas: PatternArenas + 'static>(
    depth: u32,
    max_number_of_nodes: u32,
    items_per_collection: u32,
) -> impl Strategy<Value = ArenasTo<Arenas, Index<PatternNode>>> {
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
            let make_time_cat = |total_cycle_length, multiple| {
                Pattern::TimeCat { total_cycle_length, multiple }
            };
            let make_arrange = |total_cycle_length, multiple| {
                Pattern::Arrange { total_cycle_length, multiple }
            };
            let timed_step_functions = prop_oneof![
                Just(make_time_cat as fn(_, _) -> _),
                Just(make_arrange as fn(_, _) -> _),
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
                        if let Some(time_units) = opt_time_units {
                            pattern_to_time_cat_or_arrange(
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

#[allow(unused)]
pub fn arb_small_pattern<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<PatternNode>>> {
    arb_pattern(5, 40, 7)
}

pub fn arb_large_pattern<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<PatternNode>>> {
    arb_pattern(8, 256, 10)
}
