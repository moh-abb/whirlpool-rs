use std::cmp::Ordering;

use crate::ast::PatternNode;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::ArenaTest2;
use crate::mem::arena::test::arenas_test_run;
use crate::mem::arena::test::double_arenas_test_run;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;

struct EqualToItself;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for EqualToItself
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        let lhs_adapter =
            PatternOrdAdapter::new_left(index.clone(), &shared_arena_ref);
        let rhs_adapter =
            PatternOrdAdapter::new_right(index, &shared_arena_ref);
        assert!(lhs_adapter.cmp(&rhs_adapter).is_eq())
    }
}

struct PatternEqualIffReprEqual;
impl<Arenas: PatternArenas> ArenaTest2<Index<PatternNode>, Arenas>
    for PatternEqualIffReprEqual
{
    fn run<'r>(
        arenas: SharedArenaRef<'r, Arenas>,
        pattern_x: Index<PatternNode>,
        pattern_y: Index<PatternNode>,
    ) {
        let lhs_repr = PatternDisplayAdapter::new(pattern_x.clone(), &arenas);
        let rhs_repr = PatternDisplayAdapter::new(pattern_y.clone(), &arenas);
        let lhs_adapter =
            PatternOrdAdapter::new_left(pattern_x.clone(), &arenas);
        let rhs_adapter =
            PatternOrdAdapter::new_right(pattern_y.clone(), &arenas);
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
}

#[test]
fn pattern_is_equal_to_itself() {
    arenas_test_run::<
        _,
        EqualToItself,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal() {
    double_arenas_test_run::<
        _,
        PatternEqualIffReprEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}
