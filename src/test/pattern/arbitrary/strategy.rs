use proptest::strategy::Strategy;

use crate::ast::PatternNode;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arbitrary::ArenasToPatternIndex;
use crate::ast::pattern::arbitrary::arb_large_pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::mem::arena::test::StrategyWithArena;

pub struct AnyPatternStrategy;
impl<Arenas: PatternArenas>
    StrategyWithArena<Index<PatternNode>, Arenas, ArbPatternError>
    for AnyPatternStrategy
{
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasToPatternIndex<'r, Arenas>>
    where
        Arenas: 'r,
    {
        arb_large_pattern()
    }
}
