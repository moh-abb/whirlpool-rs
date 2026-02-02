use core::mem;

use crate::structures::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::equality::PatternOrdAdapter;
use crate::ast::pattern::format_display::PatternDisplayAdapter;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::get_arena_sizes;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

struct DoNothing;
impl ArenaTest for DoNothing {
    fn run(_: &impl PatternArenas, _: Index<Pattern>) {}
}

struct DropPattern;
impl ArenaTest for DropPattern {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        mem::drop(PatternDropAdapter::new(pattern, arenas))
    }
}

struct CloneAndCheckEqual;
impl ArenaTest for CloneAndCheckEqual {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        let mut adapter = PatternCloneDropAdapter::new(pattern.clone(), arenas);
        let cloned = adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        adapter.take_item();
        let comparison = PatternOrdAdapter::new(pattern.clone(), arenas)
            .cmp(&PatternOrdAdapter::new(cloned.clone(), arenas));
        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(pattern, arenas),
            PatternDisplayAdapter::new(cloned, arenas),
        )
    }
}

struct CloneAndDropAndCheckEqual;
impl ArenaTest for CloneAndDropAndCheckEqual {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        let mut pattern_adapter =
            PatternCloneDropAdapter::new(pattern.clone(), arenas);
        let cloned = pattern_adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        pattern_adapter.take_item();
        let cloned_adapter = PatternCloneDropAdapter::new(cloned, arenas);
        let cloned_2 = cloned_adapter.clone().take_item();
        mem::drop(cloned_adapter);
        let cloned_2_adapter =
            PatternCloneDropAdapter::new(cloned_2.clone(), arenas);
        let cloned_3 = cloned_2_adapter.clone().take_item();
        mem::drop(cloned_2_adapter);
        let comparison = PatternOrdAdapter::new(cloned_3.clone(), arenas)
            .cmp(&PatternOrdAdapter::new(pattern.clone(), arenas));
        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(pattern, arenas),
            PatternDisplayAdapter::new(cloned_3, arenas),
        )
    }
}

struct CloneAndDropAndCheckSizesEqual;
impl ArenaTest for CloneAndDropAndCheckSizesEqual {
    fn run(arenas: &impl PatternArenas, pattern: Index<Pattern>) {
        let arena_sizes = get_arena_sizes(arenas);
        let sizes = arena_sizes();
        let mut clone_adapter =
            PatternCloneDropAdapter::new(pattern.clone(), arenas);
        let pattern_2 = clone_adapter.clone().take_item();
        let sizes_2 = arena_sizes();
        let size_diff_1 =
            core::array::from_fn::<_, 4, _>(|i| sizes_2[i] - sizes[i]);
        let pattern_3 = clone_adapter.clone().take_item();
        let sizes_3 = arena_sizes();
        let size_diff_2 =
            core::array::from_fn::<_, 4, _>(|i| sizes_3[i] - sizes_2[i]);
        assert_eq!(
            dbg!(size_diff_1),
            dbg!(size_diff_2),
            "Cloning twice should increase the number of elements by the same amount"
        );
        let pattern_3_adapter =
            PatternDropAdapter::new(pattern_3.clone(), arenas);
        println!(
            "Pattern 3 drop adapter: {}",
            PatternDisplayAdapter::new(pattern_3, arenas),
        );
        mem::drop(pattern_3_adapter);
        let sizes_4 = arena_sizes();
        assert_eq!(
            dbg!(sizes_4),
            dbg!(sizes_2),
            "Cloning then dropping should preserve the number of elements"
        );
        mem::drop(PatternDropAdapter::new(pattern_2, arenas));
        let sizes_5 = arena_sizes();
        assert_eq!(
            dbg!(sizes_5),
            dbg!(sizes),
            "Cloning then dropping twice should preserve the number of elements"
        );
        // To check that the behaviour of the arena is preserved even while
        // the arena is filled with elements, we do not drop the provided
        // `pattern`.
        clone_adapter.take_item();
    }
}

#[test]
fn can_allocate_in_growable_arenas_once() {
    with_regenerated_arenas::<DoNothing, GrowableArenas>()
}

#[test]
fn can_allocate_in_growable_arenas_multiple() {
    with_reused_arenas::<DoNothing, GrowableArenas>()
}

#[test]
fn can_allocate_then_deallocate_once() {
    with_regenerated_arenas::<DropPattern, GrowableArenas>()
}

#[test]
fn can_allocate_then_deallocate_multiple() {
    with_reused_arenas::<DropPattern, GrowableArenas>()
}

#[test]
fn can_clone_and_result_is_equal_once() {
    with_regenerated_arenas::<CloneAndCheckEqual, GrowableArenas>()
}

#[test]
fn can_clone_and_result_is_equal_multiple() {
    with_reused_arenas::<CloneAndCheckEqual, GrowableArenas>()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_once() {
    with_regenerated_arenas::<CloneAndDropAndCheckEqual, GrowableArenas>()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_multiple() {
    with_reused_arenas::<CloneAndDropAndCheckEqual, GrowableArenas>()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_once() {
    with_regenerated_arenas::<CloneAndDropAndCheckSizesEqual, GrowableArenas>()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_multiple() {
    with_reused_arenas::<CloneAndDropAndCheckSizesEqual, GrowableArenas>()
}
