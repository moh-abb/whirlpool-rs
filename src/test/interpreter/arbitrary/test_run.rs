use crate::ast::pattern::arenas::PatternArenas;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::arbitrary::strategy::ArbitraryLargeSequenceStrategy;
use crate::test::interpreter::arbitrary::strategy::ArbitrarySmallSequenceStrategy;
use crate::test::interpreter::arbitrary::strategy::StrategyOutput;
use crate::test::interpreter::test_expectations_with_interpreter_setup_and_start_time;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::pattern::arenas::GrowableArena<PatternNode>;

pub struct PlayedUnitsMatchExpectations;
impl<Arenas: PatternArenas> ArenaTest<StrategyOutput, Arenas>
    for PlayedUnitsMatchExpectations
{
    fn run(arenas: &Arenas, strategy_output: StrategyOutput) {
        let Ok((index, sequence)) = strategy_output else {
            panic!("Should not have overflowed when calculating expectations");
        };
        test_expectations_with_interpreter_setup_and_start_time(
            arenas,
            index,
            sequence.interval.start(),
            &[(sequence.interval.end(), sequence.expected)],
            FullInterpreterSetup {
                offset: sequence.offset,
                multiplier: sequence.multiplier,
            },
        );
    }
}

#[test]
fn played_small_pattern_units_match_expectations_once() {
    with_regenerated_arenas::<
        _,
        PlayedUnitsMatchExpectations,
        GrowableArena<PatternNode>,
        ArbitrarySmallSequenceStrategy,
    >()
}

#[test]
fn played_small_pattern_units_match_expectations_multiple() {
    with_reused_arenas::<
        _,
        PlayedUnitsMatchExpectations,
        GrowableArena<PatternNode>,
        ArbitrarySmallSequenceStrategy,
    >()
}

#[test]
fn played_large_pattern_units_match_expectations_once() {
    with_regenerated_arenas::<
        _,
        PlayedUnitsMatchExpectations,
        GrowableArena<PatternNode>,
        ArbitraryLargeSequenceStrategy,
    >()
}

#[test]
fn played_large_pattern_units_match_expectations_multiple() {
    with_reused_arenas::<
        _,
        PlayedUnitsMatchExpectations,
        GrowableArena<PatternNode>,
        ArbitraryLargeSequenceStrategy,
    >()
}
