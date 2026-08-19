use std::cmp::Ordering;

use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::ArenaTest2;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_regenerated_arenas_double;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::mem::arena_test::with_reused_arenas_double;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;

struct EqualToItself;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for EqualToItself
{
    fn run(arenas: &mut Arenas, pattern: Index<PatternNode>) {
        let lhs_adapter = PatternOrdAdapter::new_left(pattern.clone(), arenas);
        let rhs_adapter = PatternOrdAdapter::new_right(pattern, arenas);
        assert!(lhs_adapter.cmp(&rhs_adapter).is_eq())
    }
}

struct PatternEqualIffReprEqual;
impl<Arenas: PatternArenas> ArenaTest2<Index<PatternNode>, Arenas>
    for PatternEqualIffReprEqual
{
    fn run(
        arenas: &Arenas,
        pattern_x: Index<PatternNode>,
        pattern_y: Index<PatternNode>,
    ) {
        let repr1 = PatternDisplayAdapter::new(pattern_x.clone(), arenas);
        let repr2 = PatternDisplayAdapter::new(pattern_y.clone(), arenas);
        let lhs_adapter =
            PatternOrdAdapter::new_left(pattern_x.clone(), arenas);
        let rhs_adapter =
            PatternOrdAdapter::new_right(pattern_y.clone(), arenas);
        if format!("{repr1}") == format!("{repr2}") {
            assert_eq!(lhs_adapter.cmp(&rhs_adapter), Ordering::Equal)
        } else {
            assert_ne!(lhs_adapter.cmp(&rhs_adapter), Ordering::Equal)
        }
    }
}

#[test]
fn pattern_is_equal_to_itself_once() {
    with_regenerated_arenas::<
        _,
        EqualToItself,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn pattern_is_equal_to_itself_multiple() {
    with_reused_arenas::<
        _,
        EqualToItself,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal_once() {
    with_regenerated_arenas_double::<
        _,
        PatternEqualIffReprEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal_multiple() {
    with_reused_arenas_double::<
        _,
        PatternEqualIffReprEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}
