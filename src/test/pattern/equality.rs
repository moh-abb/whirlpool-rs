use std::cmp::Ordering;

use crate::structures::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::equality::PatternOrdAdapter;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::ArenaTest2;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_regenerated_arenas_double;
use crate::test::pattern::arena_alloc::with_reused_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas_double;

struct EqualToItself;
impl ArenaTest for EqualToItself {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        let lhs_adapter = PatternOrdAdapter::new(pattern.clone(), arenas);
        let rhs_adapter = PatternOrdAdapter::new(pattern, arenas);
        assert!(lhs_adapter.cmp(&rhs_adapter).is_eq())
    }
}

struct PatternEqualIffReprEqual;
impl ArenaTest2 for PatternEqualIffReprEqual {
    fn run(
        arenas: &impl PatternArenas,
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
    with_regenerated_arenas::<EqualToItself, GrowableArenas>()
}

#[test]
fn pattern_is_equal_to_itself_multiple() {
    with_reused_arenas::<EqualToItself, GrowableArenas>()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal_once() {
    with_regenerated_arenas_double::<PatternEqualIffReprEqual, GrowableArenas>()
}

#[test]
fn two_patterns_equal_if_and_only_if_display_equal_multiple() {
    with_reused_arenas_double::<PatternEqualIffReprEqual, GrowableArenas>()
}
