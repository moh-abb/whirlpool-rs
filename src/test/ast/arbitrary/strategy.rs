use proptest::strategy::Just;
use proptest::strategy::Strategy;

use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arbitrary::ArenasToPatternIndex;
use crate::ast::pattern::arbitrary::arb_large_pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::CycleTime;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Index;
use crate::mem::arena::test::ArenasTo;
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

pub struct AnyExpressionStrategy;
impl<Arenas: ExprArenas> StrategyWithArena<Index<ExprNode>, Arenas, ArenaError>
    for AnyExpressionStrategy
{
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, Index<ExprNode>, ArenaError>>
    where
        Arenas: 'r,
    {
        Just(ArenasTo::new(|arenas| {
            // TODO: Add more cases than simply an expression of Zero.
            arenas
                .clone()
                .push(ExprNode::new(Expr::Unit(ExprUnit::Number(
                    CycleTime::ZERO,
                ))))
        }))
    }
}
