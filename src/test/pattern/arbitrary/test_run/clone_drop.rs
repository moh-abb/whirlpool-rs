use core::mem;

use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::Arena;
use crate::mem::Index;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;
use crate::test::pattern::arenas::GrowableArenas;

struct DoNothing;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DoNothing
{
    fn run(_: &Arenas, _: Index<PatternNode>) {}
}

struct DropPattern;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DropPattern
{
    fn run(arenas: &Arenas, node: Index<PatternNode>) {
        mem::drop(PatternDropAdapter(Some(node), arenas))
    }
}

struct DropPatternAndCheckArenasEmpty;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DropPatternAndCheckArenasEmpty
{
    fn run(arenas: &Arenas, pattern: Index<PatternNode>) {
        let arena_size = || arenas.get_pattern_arena().size();
        assert_ne!(arena_size(), 0);
        mem::drop(PatternDropAdapter(Some(pattern), arenas));
        assert_eq!(arena_size(), 0);
    }
}

struct CloneAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndCheckEqual
{
    fn run(arenas: &Arenas, pattern: Index<PatternNode>) {
        let mut adapter = PatternCloneDropAdapter::new(pattern.clone(), arenas);
        let cloned = adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        adapter.take_item();
        let comparison = PatternOrdAdapter::new_left(pattern.clone(), arenas)
            .cmp(&PatternOrdAdapter::new_right(cloned.clone(), arenas));
        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(pattern, arenas),
            PatternDisplayAdapter::new(cloned, arenas),
        )
    }
}

struct CloneAndDropAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckEqual
{
    fn run(arenas: &Arenas, pattern: Index<PatternNode>) {
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
        let comparison = PatternOrdAdapter::new_left(cloned_3.clone(), arenas)
            .cmp(&PatternOrdAdapter::new_right(pattern.clone(), arenas));
        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(pattern, arenas),
            PatternDisplayAdapter::new(cloned_3, arenas),
        )
    }
}

struct CloneAndDropAndCheckSizesEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckSizesEqual
{
    fn run(arenas: &Arenas, pattern: Index<PatternNode>) {
        let arena_size = || arenas.get_pattern_arena().size();
        let sizes = arena_size();
        let mut clone_adapter =
            PatternCloneDropAdapter::new(pattern.clone(), arenas);
        let pattern_2 = clone_adapter.clone().take_item();
        let sizes_2 = arena_size();
        let size_diff_1 = sizes_2 - sizes;
        let pattern_3 = clone_adapter.clone().take_item();
        let sizes_3 = arena_size();
        let size_diff_2 = sizes_3 - sizes_2;
        assert_eq!(
            size_diff_1, size_diff_2,
            "Cloning twice should increase the number of elements by the same amount"
        );
        let pattern_3_adapter =
            PatternDropAdapter(Some(pattern_3.clone()), arenas);
        mem::drop(pattern_3_adapter);
        let sizes_4 = arena_size();
        assert_eq!(
            sizes_4, sizes_2,
            "Cloning then dropping should preserve the number of elements"
        );
        mem::drop(PatternDropAdapter(Some(pattern_2), arenas));
        let sizes_5 = arena_size();
        assert_eq!(
            sizes_5, sizes,
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
    with_regenerated_arenas::<_, DoNothing, GrowableArenas, AnyPatternStrategy>(
    )
}

#[test]
fn can_allocate_in_growable_arenas_multiple() {
    with_reused_arenas::<_, DoNothing, GrowableArenas, AnyPatternStrategy>()
}

#[test]
fn can_allocate_then_deallocate_once() {
    with_regenerated_arenas::<_, DropPattern, GrowableArenas, AnyPatternStrategy>(
    )
}

#[test]
fn can_allocate_then_deallocate_multiple() {
    with_reused_arenas::<_, DropPattern, GrowableArenas, AnyPatternStrategy>()
}

#[test]
fn can_allocate_then_deallocate_completely_once() {
    with_regenerated_arenas::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_allocate_then_deallocate_completely_multiple() {
    with_reused_arenas::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_result_is_equal_once() {
    with_regenerated_arenas::<
        _,
        CloneAndCheckEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_result_is_equal_multiple() {
    with_reused_arenas::<
        _,
        CloneAndCheckEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_once() {
    with_regenerated_arenas::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_multiple() {
    with_reused_arenas::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_once() {
    with_regenerated_arenas::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_multiple() {
    with_reused_arenas::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArenas,
        AnyPatternStrategy,
    >()
}
