use proptest::strategy::Strategy;

use crate::ast::expr::ExprNode;
use crate::ast::expr::arbitrary::ArbExprError;
use crate::ast::expr::arbitrary::arb_large_expr;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::Index;
use crate::mem::arena::test::ArenasTo;
use crate::mem::arena::test::StrategyWithArena;

pub struct AnyExpressionStrategy;
impl<Arenas: ExprArenas>
    StrategyWithArena<Index<ExprNode>, Arenas, ArbExprError>
    for AnyExpressionStrategy
{
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, Index<ExprNode>, ArbExprError>>
    where
        Arenas: 'r,
    {
        arb_large_expr()
    }
}
