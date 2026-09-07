#![cfg(test)]

use core::num::NonZeroU8;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::prop;
use proptest::prop_oneof;

use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::PatternArenasExt;
use crate::ast::pattern::arenas::PatternPushBackError;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::note::arbitrary::arb_note_unit;
use crate::ast::time::arbitrary::arb_positive_cycle_time;
use crate::mem::ArenaError;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenasTo;
use crate::mem::linked::checker::check_acyclic;

#[allow(unused)]
#[derive(derive_more::From, Debug)]
pub enum ArbPatternError {
    ArenaErr(ArenaError),
    PushBackErr(PatternPushBackError),
}

fn arb_pattern_leaf() -> impl Strategy<Value = Pattern> {
    prop_oneof![Just(Pattern::Silence), arb_note_unit().prop_map(Pattern::Note)]
}

pub type ArenasToPatternDropAdapter<'r, Arenas> = ArenasTo<
    'r,
    Arenas,
    PatternDropAdapter<SharedArenaRef<'r, Arenas>>,
    ArbPatternError,
>;
pub type ArenasToPatternIndex<'r, Arenas> =
    ArenasTo<'r, Arenas, Index<PatternNode>, ArbPatternError>;

fn pattern_to_time_cat_or_arrange<'r, Arenas: PatternArenas + 'r>(
    xs: Vec<ArenasToPatternDropAdapter<'r, Arenas>>,
    time_units: Vec<CycleTime>,
    f: fn(CycleTime, Multiple<PatternNode>) -> Pattern,
) -> ArenasToPatternDropAdapter<'r, Arenas> {
    let total_cycle_time = time_units
        .iter()
        .cloned()
        .sum::<Result<CycleTime, _>>()
        .expect("expected times to not overflow");

    ArenasTo::new(move |arenas: SharedArenaRef<'r, Arenas>| {
        let result_adapter = arenas
            .clone()
            .push_dropping(f(total_cycle_time, Multiple::new()))?;
        let result_index = result_adapter.clone_index().unwrap();

        xs.iter()
            .cloned()
            .zip(&time_units)
            .try_for_each(|(x, duration)| {
                // Obtain the arbitrary pattern which will form
                // the TimedStep.
                let child_adapter = x.call(arenas.clone())?;
                arenas.clone().push_timed_step_adapter(
                    result_index.clone(),
                    *duration,
                    child_adapter,
                )?;
                Result::<_, ArbPatternError>::Ok(())
            })?;

        check_acyclic(result_index.clone(), &arenas);
        Ok(result_adapter)
    })
}

fn pattern_to_multiple_pattern<'r, Arenas: PatternArenas + 'r>(
    xs: Vec<ArenasToPatternDropAdapter<'r, Arenas>>,
    f: fn(Multiple<PatternNode>) -> Pattern,
) -> ArenasToPatternDropAdapter<'r, Arenas> {
    ArenasTo::new(move |arenas: SharedArenaRef<'r, Arenas>| {
        let result_adapter = arenas
            .clone()
            .push_dropping(f(Multiple::new()))?;

        let result_index = result_adapter.clone_index().unwrap();

        xs.iter().cloned().try_for_each(|x| {
            let child_adapter = x.call(arenas.clone())?;
            arenas
                .clone()
                .push_pattern_adapter(result_index.clone(), child_adapter)?;
            Result::<_, ArbPatternError>::Ok(())
        })?;

        check_acyclic(result_index.clone(), &arenas);
        Ok(result_adapter)
    })
}

fn arenas_to_index<'r, Arenas: PatternArenas + 'r>(
    arenas_to_adapter: ArenasToPatternDropAdapter<'r, Arenas>,
) -> ArenasToPatternIndex<'r, Arenas> {
    ArenasTo::new(move |arenas| {
        let mut adapter = arenas_to_adapter.call(arenas)?;
        Ok(adapter.take_index().unwrap())
    })
}

/// Used as a workaround since `prop_recursive` requires a strategy that has
/// `'static` lifetime, but since we are dealing with values that may borrow
/// arenas with a shorter lifetime, we need to first create the tree with
/// placeholder values and then traverse it again layer-by-layer to get the
/// final tree.
#[derive(Debug, Clone)]
enum PlaceholderTree {
    Leaf(Pattern),
    NormalPattern {
        children: Vec<Self>,
        func: fn(Multiple<PatternNode>) -> Pattern,
    },
    TimedStepPattern {
        children: Vec<Self>,
        time_units: Vec<CycleTime>,
        func: fn(CycleTime, Multiple<PatternNode>) -> Pattern,
    },
}

fn arb_placeholder_tree(
    depth: u32,
    max_number_of_nodes: u32,
    items_per_collection: u32,
) -> impl Strategy<Value = PlaceholderTree> {
    arb_pattern_leaf()
        .prop_map(PlaceholderTree::Leaf)
        .prop_recursive(
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
                        |(
                            children,
                            opt_time_units,
                            pattern_func,
                            timed_step_func,
                        )| {
                            if let Some(time_units) = opt_time_units {
                                PlaceholderTree::TimedStepPattern {
                                    children,
                                    time_units,
                                    func: timed_step_func,
                                }
                            } else {
                                PlaceholderTree::NormalPattern {
                                    children,
                                    func: pattern_func,
                                }
                            }
                        },
                    )
            },
        )
}

fn placeholder_tree_to_pattern_tree<'r, Arenas: PatternArenas + 'r>(
    tree: PlaceholderTree,
) -> ArenasToPatternDropAdapter<'r, Arenas> {
    let raw_placeholders_to_adapters = |children: Vec<PlaceholderTree>| {
        children
            .into_iter()
            .map(placeholder_tree_to_pattern_tree)
            .collect::<Vec<_>>()
    };

    match tree {
        PlaceholderTree::Leaf(pattern) => ArenasTo::new(move |arenas| {
            Ok(arenas
                .clone()
                .push_dropping(pattern.clone())?)
        }),
        PlaceholderTree::NormalPattern { children, func } => {
            pattern_to_multiple_pattern(
                raw_placeholders_to_adapters(children),
                func,
            )
        }
        PlaceholderTree::TimedStepPattern { children, time_units, func } => {
            pattern_to_time_cat_or_arrange(
                raw_placeholders_to_adapters(children),
                time_units,
                func,
            )
        }
    }
}

fn arb_pattern<'r, Arenas: PatternArenas + 'r>(
    depth: u32,
    max_number_of_nodes: u32,
    items_per_collection: u32,
) -> impl Strategy<Value = ArenasToPatternIndex<'r, Arenas>> + 'r {
    arb_placeholder_tree(depth, max_number_of_nodes, items_per_collection)
        .prop_map(placeholder_tree_to_pattern_tree)
        .prop_map(arenas_to_index)
}

#[allow(unused)]
pub fn arb_small_pattern<'r, Arenas: PatternArenas + 'r>()
-> impl Strategy<Value = ArenasToPatternIndex<'r, Arenas>> {
    arb_pattern(5, 40, 7)
}

pub fn arb_large_pattern<'r, Arenas: PatternArenas + 'r>()
-> impl Strategy<Value = ArenasToPatternIndex<'r, Arenas>> {
    arb_pattern(8, 256, 10)
}
