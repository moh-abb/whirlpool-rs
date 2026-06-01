use std::cmp::Ordering;

use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::mem::Index;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::ArenaTest2;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_regenerated_arenas_double;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::mem::arena_test::with_reused_arenas_double;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;
use crate::test::pattern::arenas::GrowableArenas;

struct EqualToItself;
impl<Arenas: PatternArenas> ArenaTest<Index<Pattern>, Arenas>
    for EqualToItself
{
    fn run(arenas: &Arenas, pattern: Index<Pattern>) {
        let lhs_adapter = PatternOrdAdapter::new(pattern.clone(), arenas);
        let rhs_adapter = PatternOrdAdapter::new(pattern, arenas);
        assert!(lhs_adapter.cmp(&rhs_adapter).is_eq())
    }
}

struct PatternEqualIffReprEqual;
impl<Arenas: PatternArenas> ArenaTest2<Index<Pattern>, Arenas>
    for PatternEqualIffReprEqual
{
    fn run(
        arenas: &Arenas,
        pattern1: Index<Pattern>,
        pattern2: Index<Pattern>,
    ) {
        let repr1 = PatternDisplayAdapter::new(pattern1.clone(), arenas);
        let repr2 = PatternDisplayAdapter::new(pattern2.clone(), arenas);
        let lhs_adapter = PatternOrdAdapter::new(pattern1.clone(), arenas);
        let rhs_adapter = PatternOrdAdapter::new(pattern2.clone(), arenas);
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
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn pattern_is_equal_to_itself_multiple() {
    with_reused_arenas::<_, EqualToItself, GrowableArenas, AnyPatternStrategy>()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal_once() {
    with_regenerated_arenas_double::<
        _,
        PatternEqualIffReprEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal_multiple() {
    with_reused_arenas_double::<
        _,
        PatternEqualIffReprEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}
