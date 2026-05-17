use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::ast::pattern::string_display::PatternDisplayVisitorL;
use crate::ast::pattern::string_display::PatternDisplayVisitorR;
use crate::mem::Index;
use crate::test::pattern::arena_alloc::AnyPatternStrategy;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

struct LeftAndRightDisplaysEqual;
impl ArenaTest<Index<Pattern>> for LeftAndRightDisplaysEqual {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
            PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
        )
    }
}

struct LeftDisplayEqualToFormatted;
impl ArenaTest<Index<Pattern>> for LeftDisplayEqualToFormatted {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
        )
    }
}

struct RightDisplayEqualToFormatted;
impl ArenaTest<Index<Pattern>> for RightDisplayEqualToFormatted {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
        )
    }
}

#[test]
fn left_and_right_displays_are_equal_once() {
    with_regenerated_arenas::<
        _,
        LeftAndRightDisplaysEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn left_and_right_displays_are_equal_multiple() {
    with_reused_arenas::<
        _,
        LeftAndRightDisplaysEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn left_display_equal_to_format_display_once() {
    with_regenerated_arenas::<
        _,
        LeftDisplayEqualToFormatted,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn left_display_equal_to_format_display_multiple() {
    with_reused_arenas::<
        _,
        LeftDisplayEqualToFormatted,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn right_display_equal_to_format_display_once() {
    with_regenerated_arenas::<
        _,
        RightDisplayEqualToFormatted,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn right_display_equal_to_format_display_multiple() {
    with_reused_arenas::<
        _,
        RightDisplayEqualToFormatted,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}
