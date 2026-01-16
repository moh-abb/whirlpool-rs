use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::ast::pattern::string_display::PatternDisplayVisitorL;
use crate::ast::pattern::string_display::PatternDisplayVisitorR;
use crate::test::pattern::arena_alloc::TesterFn;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

const LEFT_AND_RIGHT_DISPLAYS_EQUAL: TesterFn = |arenas, pattern| {
    assert_eq!(
        PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
        PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
    )
};

const LEFT_DISPLAY_EQUAL_TO_FORMAT_DISPLAY: TesterFn = |arenas, pattern| {
    assert_eq!(
        format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
        PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
    )
};

const RIGHT_DISPLAY_EQUAL_TO_FORMAT_DISPLAY: TesterFn = |arenas, pattern| {
    assert_eq!(
        format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
        PatternDisplayVisitorR::new(arenas).display(pattern.clone()),
    )
};

#[test]
fn left_and_right_displays_are_equal_once() {
    with_regenerated_arenas(LEFT_AND_RIGHT_DISPLAYS_EQUAL)
}

#[test]
fn left_and_right_displays_are_equal_multiple() {
    with_reused_arenas(LEFT_AND_RIGHT_DISPLAYS_EQUAL)
}

#[test]
fn left_display_equal_to_format_display_once() {
    with_regenerated_arenas(LEFT_DISPLAY_EQUAL_TO_FORMAT_DISPLAY)
}

#[test]
fn left_display_equal_to_format_display_multiple() {
    with_reused_arenas(LEFT_DISPLAY_EQUAL_TO_FORMAT_DISPLAY)
}

#[test]
fn right_display_equal_to_format_display_once() {
    with_regenerated_arenas(RIGHT_DISPLAY_EQUAL_TO_FORMAT_DISPLAY)
}

#[test]
fn right_display_equal_to_format_display_multiple() {
    with_reused_arenas(RIGHT_DISPLAY_EQUAL_TO_FORMAT_DISPLAY)
}
