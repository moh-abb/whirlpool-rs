use crate::arena::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::ast::pattern::string_display::PatternDisplayVisitorL;
use crate::ast::pattern::string_display::PatternDisplayVisitorR;
use crate::test::pattern::arena_alloc::ArenaTest;
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

struct LeftDisplayEqualToFormatDisplay;
impl ArenaTest for LeftDisplayEqualToFormatDisplay {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
        )
    }
}

struct RightDisplayEqualToFormatDisplay;
impl ArenaTest for RightDisplayEqualToFormatDisplay {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
        )
    }
}

#[test]
fn left_and_right_displays_are_equal_once() {
    with_regenerated_arenas::<LeftAndRightDisplaysEqual>()
}

#[test]
fn left_and_right_displays_are_equal_multiple() {
    with_reused_arenas::<LeftAndRightDisplaysEqual>()
}

#[test]
fn left_display_equal_to_format_display_once() {
    with_regenerated_arenas::<LeftDisplayEqualToFormatDisplay>()
}

#[test]
fn left_display_equal_to_format_display_multiple() {
    with_reused_arenas::<LeftDisplayEqualToFormatDisplay>()
}

#[test]
fn right_display_equal_to_format_display_once() {
    with_regenerated_arenas::<RightDisplayEqualToFormatDisplay>()
}

#[test]
fn right_display_equal_to_format_display_multiple() {
    with_reused_arenas::<RightDisplayEqualToFormatDisplay>()
}
