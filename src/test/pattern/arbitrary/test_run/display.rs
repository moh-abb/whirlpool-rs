use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::ast::pattern::string_display::PatternDisplayVisitor;
use crate::mem::Index;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;
use crate::test::pattern::arenas::GrowableArenas;

struct LeftAndRightDisplaysEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<Pattern>, Arenas>
    for LeftAndRightDisplaysEqual
{
    fn run(arenas: &Arenas, pattern: Index<Pattern>) {
        assert_eq!(
            PatternDisplayVisitor::new_left(arenas)
                .display(pattern.clone())
                .unwrap(),
            PatternDisplayVisitor::new_right(arenas)
                .display(pattern.clone())
                .unwrap(),
        )
    }
}

struct LeftDisplayEqualToFormatted;
impl<Arenas: PatternArenas> ArenaTest<Index<Pattern>, Arenas>
    for LeftDisplayEqualToFormatted
{
    fn run(arenas: &Arenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitor::new_left(arenas)
                .display(pattern.clone())
                .unwrap(),
        )
    }
}

struct RightDisplayEqualToFormatted;
impl<Arenas: PatternArenas> ArenaTest<Index<Pattern>, Arenas>
    for RightDisplayEqualToFormatted
{
    fn run(arenas: &Arenas, pattern: Index<Pattern>) {
        assert_eq!(
            format!("{}", PatternDisplayAdapter::new(pattern.clone(), arenas)),
            PatternDisplayVisitor::new_right(arenas)
                .display(pattern.clone())
                .unwrap(),
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
