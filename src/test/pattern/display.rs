use crate::ast::pattern::display::PatternDisplayVisitorL;
use crate::ast::pattern::display::PatternDisplayVisitorR;
use crate::test::pattern::arena_alloc::TesterFn;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

const LEFT_AND_RIGHT_DISPLAYS_EQUAL: TesterFn = |arenas, pattern| {
    assert_eq!(
        PatternDisplayVisitorL::new(arenas).display(pattern.clone()),
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
