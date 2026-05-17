use proptest::prelude::Strategy;

use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::test::interpreter::arbitrary::expectations::arb_interval_offset_and_multiplier;
use crate::test::interpreter::arbitrary::expectations::pattern_expectations;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::pattern::arb_small_pattern;
use crate::test::pattern::arena_alloc::StrategyWithArena;

pub struct ArbitrarySequenceStrategy;
impl StrategyWithArena<(Index<Pattern>, NoteSequence)>
    for ArbitrarySequenceStrategy
{
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, (Index<Pattern>, NoteSequence)>>
    {
        (arb_small_pattern(), arb_interval_offset_and_multiplier()).prop_map(
            |(arenas_to_pattern, (interval, offset, multiplier))| {
                ArenasTo::new(move |arenas: &Arenas| {
                    let pattern = arenas_to_pattern.call(arenas)?;
                    let expectations = pattern_expectations(
                        pattern.clone(),
                        arenas,
                        interval,
                        offset,
                        multiplier,
                    )?;
                    Ok((pattern, expectations))
                })
            },
        )
    }
}
