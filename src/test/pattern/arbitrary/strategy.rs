use proptest::strategy::Strategy;

use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::test::mem::arena_test::StrategyWithArena;
use crate::test::mem::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::pattern::arb_large_pattern;

pub struct AnyPatternStrategy;
impl<Arenas: PatternArenas + 'static> StrategyWithArena<Index<Pattern>, Arenas>
    for AnyPatternStrategy
{
    fn item_strategy() -> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>>
    {
        arb_large_pattern()
    }
}
