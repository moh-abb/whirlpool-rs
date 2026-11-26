use core::fmt::Display;
use std::cmp::Ordering;

use crate::ast::expr::ExprNode;
use crate::ast::expr::arbitrary::ArbExprError;
use crate::ast::expr::cmp::ExprOrdAdapter;
use crate::ast::expr::format::ExprDisplayAdapter;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::ArenaTest2;
use crate::mem::arena::test::arenas_test_run;
use crate::mem::arena::test::double_arenas_test_run;
use crate::test::ast::arbitrary::strategy::AnyExpressionStrategy;

fn check_nodes_equal_to_itself<
    'r,
    'a: 'r,
    Node,
    Arenas: 'a,
    OrdAdapter: Ord,
>(
    shared_arena_ref: &'r SharedArenaRef<'a, Arenas>,
    index: Index<Node>,
    make_ord_adapter_left: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_ord_adapter_right: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
) {
    let lhs_adapter = make_ord_adapter_left(index.clone(), shared_arena_ref);
    let rhs_adapter = make_ord_adapter_right(index, shared_arena_ref);
    assert!(lhs_adapter.cmp(&rhs_adapter).is_eq())
}

struct EqualToItself;
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas> for EqualToItself {
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<ExprNode>,
    ) {
        check_nodes_equal_to_itself(
            &shared_arena_ref,
            index,
            ExprOrdAdapter::new_left,
            ExprOrdAdapter::new_right,
        );
    }
}

fn check_nodes_equal_iff_repr_equal<
    'r,
    'a: 'r,
    Node,
    Arenas: 'a,
    OrdAdapter: Ord,
    DisplayAdapter: Display,
>(
    shared_arena_ref: &'r SharedArenaRef<'a, Arenas>,
    index_x: Index<Node>,
    index_y: Index<Node>,
    make_ord_adapter_left: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_ord_adapter_right: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_display_adapter: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> DisplayAdapter,
) {
    let lhs_repr = make_display_adapter(index_x.clone(), shared_arena_ref);
    let rhs_repr = make_display_adapter(index_y.clone(), shared_arena_ref);
    let lhs_adapter = make_ord_adapter_left(index_x.clone(), shared_arena_ref);
    let rhs_adapter = make_ord_adapter_right(index_y.clone(), shared_arena_ref);
    let lhs_fmt = format!("{lhs_repr}");
    let rhs_fmt = format!("{rhs_repr}");
    if lhs_fmt == rhs_fmt {
        assert_eq!(
            lhs_adapter.cmp(&rhs_adapter),
            Ordering::Equal,
            "LHS {lhs_fmt} and RHS {rhs_fmt} compared differently
            but representations are the same"
        )
    } else {
        assert_ne!(
            lhs_adapter.cmp(&rhs_adapter),
            Ordering::Equal,
            "LHS {lhs_fmt} and RHS {rhs_fmt} compared equally
            but representations are different"
        )
    }
}

struct EqualIffReprEqual;
impl<Arenas: ExprArenas> ArenaTest2<Index<ExprNode>, Arenas>
    for EqualIffReprEqual
{
    fn run<'r>(
        arenas: SharedArenaRef<'r, Arenas>,
        index_x: Index<ExprNode>,
        index_y: Index<ExprNode>,
    ) {
        check_nodes_equal_iff_repr_equal(
            &arenas,
            index_x,
            index_y,
            ExprOrdAdapter::new_left,
            ExprOrdAdapter::new_right,
            ExprDisplayAdapter::new,
        );
    }
}

#[test]
fn expression_is_equal_to_itself() {
    arenas_test_run::<
        _,
        EqualToItself,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArbExprError,
    >()
}

#[test]
fn two_expressions_equal_if_and_only_if_display_equal() {
    double_arenas_test_run::<
        _,
        EqualIffReprEqual,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArbExprError,
    >()
}
