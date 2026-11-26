use core::cmp::Ordering;
use core::fmt::Debug;

use combine::EasyParser;

use crate::ast::expr::ExprNode;
use crate::ast::expr::adapter::ExprArenaAdapter;
use crate::ast::expr::arbitrary::ArbExprError;
use crate::ast::expr::cmp::ExprOrdAdapter;
use crate::ast::expr::format::ExprDisplayAdapter;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::arenas_test_run;
use crate::test::ast::arbitrary::strategy::AnyExpressionStrategy;

struct ValidateExpr;
impl<Arenas: ExprArenas + Debug> ArenaTest<Index<ExprNode>, Arenas>
    for ValidateExpr
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        orig_index: Index<ExprNode>,
    ) {
        let display_adapter =
            ExprDisplayAdapter::new(orig_index.clone(), &shared_arena_ref);
        let orig_formatted = format!("{display_adapter}");
        let mut parser = ExprArenaAdapter::parser(shared_arena_ref.clone());
        let input = orig_formatted.as_bytes();
        let mut result_adapter = parser
            .easy_parse(input)
            .map_err(|err| err.map_position(|p| p.translate_position(input)))
            .unwrap_or_else(|e| {
                panic!(
                    "Parsing Expr {orig_formatted} \
                    failed with error {e:?}"
                )
            })
            .0
            .unwrap_or_else(|e| {
                panic!(
                    "Parsed Expr {orig_formatted} \
                    contains error {e:?}"
                )
            });
        let result_index = result_adapter
            .take_index()
            .expect("Adapter should not store an error")
            .expect("Should have a full item inside the adapter");
        let result_formatted =
            ExprDisplayAdapter::new(result_index.clone(), &shared_arena_ref);

        assert_eq!(
            ExprOrdAdapter::new_left(orig_index, &shared_arena_ref).cmp(
                &ExprOrdAdapter::new_right(result_index, &shared_arena_ref)
            ),
            Ordering::Equal,
            "Parsed Expr {result_formatted} should be
            equal to original {orig_formatted}"
        );
    }
}

#[test]
fn can_parse_expr() {
    arenas_test_run::<
        Index<ExprNode>,
        ValidateExpr,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArbExprError,
    >();
}
