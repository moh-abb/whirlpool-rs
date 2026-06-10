use proptest::prelude::Strategy;

use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::OverflowError;
use crate::mem::Index;
use crate::test::examples::arbitrary::pattern::arb_large_pattern;
use crate::test::examples::arbitrary::pattern::arb_small_pattern;
use crate::test::interpreter::arbitrary::expectations::ExpectationError;
use crate::test::interpreter::arbitrary::expectations::arb_interval_offset_and_multiplier;
use crate::test::interpreter::arbitrary::expectations::pattern_expectations;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::mem::arena_test::StrategyWithArena;
use crate::test::mem::arenas_to::ArenasTo;

pub type StrategyOutput = Result<(Index<Pattern>, NoteSequence), OverflowError>;

fn prop_map_func<Arenas: PatternArenas + 'static>(
    (arenas_to_pattern, (interval, offset, multiplier)): (
        ArenasTo<Arenas, Index<Pattern>>,
        (
            crate::ast::CycleInterval,
            crate::ast::CycleTime,
            crate::ast::CycleTime,
        ),
    ),
) -> ArenasTo<Arenas, Result<(Index<Pattern>, NoteSequence), OverflowError>> {
    ArenasTo::new(move |arenas: &Arenas| {
        let pattern = arenas_to_pattern.call(arenas)?;
        let opt_expectations = pattern_expectations(
            pattern.clone(),
            arenas,
            interval,
            offset,
            multiplier,
        );
        match opt_expectations {
            Ok(expectations) => Ok(Ok((pattern, expectations))),
            Err(ExpectationError::ArenaErr(e)) => Err(e),
            Err(ExpectationError::OverflowErr(e)) => Ok(Err(e)),
        }
    })
}

pub struct ArbitraryLargeSequenceStrategy;
impl<Arenas: PatternArenas + 'static> StrategyWithArena<StrategyOutput, Arenas>
    for ArbitraryLargeSequenceStrategy
{
    fn item_strategy() -> impl Strategy<Value = ArenasTo<Arenas, StrategyOutput>>
    {
        (arb_large_pattern(), arb_interval_offset_and_multiplier())
            .prop_map(prop_map_func)
    }
}

pub struct ArbitrarySmallSequenceStrategy;
impl<Arenas: PatternArenas + 'static> StrategyWithArena<StrategyOutput, Arenas>
    for ArbitrarySmallSequenceStrategy
{
    fn item_strategy() -> impl Strategy<Value = ArenasTo<Arenas, StrategyOutput>>
    {
        (arb_small_pattern(), arb_interval_offset_and_multiplier())
            .prop_map(prop_map_func)
    }
}
