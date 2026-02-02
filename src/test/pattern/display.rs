use crate::structures::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::ast::pattern::string_display::PatternDisplayVisitorL;
use crate::ast::pattern::string_display::PatternDisplayVisitorR;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

struct LeftAndRightDisplaysEqual;
impl ArenaTest for LeftAndRightDisplaysEqual {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
            PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
        )
    }
}

struct LeftDisplayEqualToFormatted;
impl ArenaTest for LeftDisplayEqualToFormatted {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
        )
    }
}

struct RightDisplayEqualToFormatted;
impl ArenaTest for RightDisplayEqualToFormatted {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
        )
    }
}

#[test]
fn left_and_right_displays_are_equal_once() {
    with_regenerated_arenas::<LeftAndRightDisplaysEqual, GrowableArenas>()
}

#[test]
fn left_and_right_displays_are_equal_multiple() {
    with_reused_arenas::<LeftAndRightDisplaysEqual, GrowableArenas>()
}

#[test]
fn left_display_equal_to_format_display_once() {
    with_regenerated_arenas::<LeftDisplayEqualToFormatted, GrowableArenas>()
}

#[test]
fn left_display_equal_to_format_display_multiple() {
    with_reused_arenas::<LeftDisplayEqualToFormatted, GrowableArenas>()
}

#[test]
fn right_display_equal_to_format_display_once() {
    with_regenerated_arenas::<RightDisplayEqualToFormatted, GrowableArenas>()
}

#[test]
fn right_display_equal_to_format_display_multiple() {
    with_reused_arenas::<RightDisplayEqualToFormatted, GrowableArenas>()
}
