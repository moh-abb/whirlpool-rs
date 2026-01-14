use crate::arena::Arena;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::display::PatternDisplayVisitorR;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::equality::PatternOrdAdapter;
use crate::test::pattern::arena_alloc::TesterFn;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

const DO_NOTHING: TesterFn = |_, _| ();
const DROP_PATTERN: TesterFn =
    |arenas, pattern| core::mem::drop(PatternDropAdapter::new(pattern, arenas));
const CLONE_AND_CHECK_EQUAL: TesterFn = |arenas, pattern| {
    let display_visitor = PatternDisplayVisitorR::new(arenas);
    let mut adapter = PatternCloneDropAdapter::new(pattern.clone(), arenas);
    let cloned = adapter.clone().take_index();
    // To avoid dropping the pattern, take the adapter's index.
    adapter.take_index();
    let comparison = PatternOrdAdapter::new(pattern.clone(), arenas)
        .cmp(&PatternOrdAdapter::new(cloned.clone(), arenas));
    assert!(
        comparison.is_eq(),
        "Pattern {:?} and cloned {:?} are distinct",
        display_visitor.display(pattern),
        display_visitor.display(cloned),
    )
};
const CLONE_AND_DROP_AND_CHECK_EQUAL: TesterFn = |arenas, pattern| {
    let mut pattern_adapter =
        PatternCloneDropAdapter::new(pattern.clone(), arenas);
    let cloned = pattern_adapter.clone().take_index();
    // To avoid dropping the pattern, take the adapter's index.
    pattern_adapter.take_index();
    let cloned_adapter = PatternCloneDropAdapter::new(cloned, arenas);
    let cloned_2 = cloned_adapter.clone().take_index();
    core::mem::drop(cloned_adapter);
    let cloned_2_adapter =
        PatternCloneDropAdapter::new(cloned_2.clone(), arenas);
    let cloned_3 = cloned_2_adapter.clone().take_index();
    core::mem::drop(cloned_2_adapter);
    let comparison = PatternOrdAdapter::new(cloned_3.clone(), arenas)
        .cmp(&PatternOrdAdapter::new(pattern.clone(), arenas));
    assert!(
        comparison.is_eq(),
        "Pattern {:?} and cloned {:?} are distinct",
        PatternDisplayVisitorR::new(arenas).display(pattern),
        PatternDisplayVisitorR::new(arenas).display(cloned_3),
    )
};
const CLONE_AND_DROP_AND_CHECK_SIZES_EQUAL: TesterFn = |arenas, pattern| {
    let arena_sizes =
        || [arenas.0.size(), arenas.1.size(), arenas.2.size(), arenas.3.size()];
    let sizes = arena_sizes();
    let mut clone_adapter =
        PatternCloneDropAdapter::new(pattern.clone(), arenas);
    let pattern_2 = clone_adapter.clone().take_index();
    let sizes_2 = arena_sizes();
    let size_diff_1 = std::array::from_fn::<_, 4, _>(|i| sizes_2[i] - sizes[i]);
    let pattern_3 = clone_adapter.clone().take_index();
    let sizes_3 = arena_sizes();
    let size_diff_2 =
        std::array::from_fn::<_, 4, _>(|i| sizes_3[i] - sizes_2[i]);
    assert_eq!(
        dbg!(size_diff_1),
        dbg!(size_diff_2),
        "Cloning twice should increase the number of elements by the same amount"
    );
    let pattern_3_adapter = PatternDropAdapter::new(pattern_3.clone(), arenas);
    println!(
        "Pattern 3 drop adapter: {:?}",
        PatternDisplayVisitorR::new(arenas).display(pattern_3)
    );
    core::mem::drop(pattern_3_adapter);
    let sizes_4 = arena_sizes();
    assert_eq!(
        dbg!(sizes_4),
        dbg!(sizes_2),
        "Cloning then dropping should preserve the number of elements"
    );
    core::mem::drop(PatternDropAdapter::new(pattern_2, arenas));
    let sizes_5 = arena_sizes();
    assert_eq!(
        dbg!(sizes_5),
        dbg!(sizes),
        "Cloning then dropping twice should preserve the number of elements"
    );
    // To check that the behaviour of the arena is preserved even while
    // the arena is filled with elements, we do not drop the provided
    // `pattern`.
    clone_adapter.take_index();
};

#[test]
fn can_allocate_in_growable_arenas_once() {
    with_regenerated_arenas(DO_NOTHING)
}

#[test]
fn can_allocate_in_growable_arenas_multiple() {
    with_reused_arenas(DO_NOTHING)
}

#[test]
fn can_allocate_then_deallocate_once() {
    with_regenerated_arenas(DROP_PATTERN)
}

#[test]
fn can_allocate_then_deallocate_multiple() {
    with_reused_arenas(DROP_PATTERN)
}

#[test]
fn can_clone_and_result_is_equal_once() {
    with_regenerated_arenas(CLONE_AND_CHECK_EQUAL)
}

#[test]
fn can_clone_and_result_is_equal_multiple() {
    with_reused_arenas(CLONE_AND_CHECK_EQUAL)
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_once() {
    with_regenerated_arenas(CLONE_AND_DROP_AND_CHECK_EQUAL);
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_multiple() {
    with_reused_arenas(CLONE_AND_DROP_AND_CHECK_EQUAL);
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_once() {
    with_regenerated_arenas(CLONE_AND_DROP_AND_CHECK_SIZES_EQUAL);
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_multiple() {
    with_reused_arenas(CLONE_AND_DROP_AND_CHECK_SIZES_EQUAL);
}
