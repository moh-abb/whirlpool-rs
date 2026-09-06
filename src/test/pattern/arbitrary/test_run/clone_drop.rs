use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::Arena;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::arena::arena_impl::shared_arena::SharedArena;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;

struct DoNothing;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DoNothing
{
    fn run(_: &mut Arenas, _: Index<PatternNode>) {}
}

struct DropPattern;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DropPattern
{
    fn run(arenas: &mut Arenas, node: Index<PatternNode>) {
        let shared_arena = SharedArena::new(arenas);
        let shared_arena_ref = shared_arena.make_ref();
        drop(PatternDropAdapter::new(node, shared_arena_ref))
    }
}

struct DropPatternAndCheckArenasEmpty;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DropPatternAndCheckArenasEmpty
{
    fn run(arenas: &mut Arenas, pattern: Index<PatternNode>) {
        assert_ne!(arenas.size(), Ok(0));
        let shared_arena = SharedArena::new(&mut *arenas);
        let shared_arena_ref = shared_arena.make_ref();
        drop(PatternDropAdapter::new(pattern, shared_arena_ref));
        assert_eq!(arenas.size(), Ok(0));
    }
}

struct CloneAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndCheckEqual
{
    fn run(arenas: &mut Arenas, pattern: Index<PatternNode>) {
        let shared_arenas = SharedArena::new(arenas);
        let adapter_ref = shared_arenas.make_ref();
        let mut adapter =
            PatternCloneDropAdapter::new(pattern.clone(), adapter_ref);
        let cloned = adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        adapter.take_item();
        let comparison =
            PatternOrdAdapter::new_left(pattern.clone(), &adapter_ref).cmp(
                &PatternOrdAdapter::new_right(cloned.clone(), &adapter_ref),
            );
        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(pattern, &adapter_ref),
            PatternDisplayAdapter::new(cloned, &adapter_ref),
        )
    }
}

struct CloneAndDropAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckEqual
{
    fn run(arenas: &mut Arenas, pattern: Index<PatternNode>) {
        let shared_arenas = SharedArena::new(arenas);
        let adapter_ref = shared_arenas.make_ref();
        let mut pattern_adapter =
            PatternCloneDropAdapter::new(pattern.clone(), adapter_ref);
        let cloned = pattern_adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        pattern_adapter.take_item();
        let cloned_adapter = PatternCloneDropAdapter::new(cloned, adapter_ref);
        let cloned_2 = cloned_adapter.clone().take_item();
        drop(cloned_adapter);
        let cloned_2_adapter =
            PatternCloneDropAdapter::new(cloned_2.clone(), adapter_ref);
        let cloned_3 = cloned_2_adapter.clone().take_item();
        drop(cloned_2_adapter);
        let comparison =
            PatternOrdAdapter::new_left(cloned_3.clone(), &adapter_ref).cmp(
                &PatternOrdAdapter::new_right(pattern.clone(), &adapter_ref),
            );
        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(pattern, &adapter_ref),
            PatternDisplayAdapter::new(cloned_3, &adapter_ref),
        )
    }
}

struct CloneAndDropAndCheckSizesEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckSizesEqual
{
    fn run(arenas: &mut Arenas, pattern: Index<PatternNode>) {
        let shared_arenas = SharedArena::new(arenas);
        let adapter_ref = shared_arenas.make_ref();
        let sizes = adapter_ref.size().unwrap();
        let mut clone_adapter =
            PatternCloneDropAdapter::new(pattern.clone(), adapter_ref);
        let pattern_2 = clone_adapter.clone().take_item();
        let sizes_2 = adapter_ref.size().unwrap();
        let size_diff_1 = sizes_2 - sizes;
        let pattern_3 = clone_adapter.clone().take_item();
        let sizes_3 = adapter_ref.size().unwrap();
        let size_diff_2 = sizes_3 - sizes_2;
        assert_eq!(
            size_diff_1, size_diff_2,
            "Cloning twice should increase the number of elements by the same amount"
        );
        let drop_ref = shared_arenas.make_ref();
        let pattern_3_adapter =
            PatternDropAdapter::new(pattern_3.clone(), drop_ref);
        drop(pattern_3_adapter);
        let sizes_4 = adapter_ref.size().unwrap();
        assert_eq!(
            sizes_4, sizes_2,
            "Cloning then dropping should preserve the number of elements"
        );
        let drop_ref_2 = shared_arenas.make_ref();
        drop(PatternDropAdapter::new(pattern_2, drop_ref_2));
        let sizes_5 = adapter_ref.size().unwrap();
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
    with_regenerated_arenas::<
        _,
        DoNothing,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_allocate_in_growable_arenas_multiple() {
    with_reused_arenas::<
        _,
        DoNothing,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_allocate_then_deallocate_once() {
    with_regenerated_arenas::<
        _,
        DropPattern,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_allocate_then_deallocate_multiple() {
    with_reused_arenas::<
        _,
        DropPattern,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_allocate_then_deallocate_completely_once() {
    with_regenerated_arenas::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_allocate_then_deallocate_completely_multiple() {
    with_reused_arenas::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_result_is_equal_once() {
    with_regenerated_arenas::<
        _,
        CloneAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_result_is_equal_multiple() {
    with_reused_arenas::<
        _,
        CloneAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_once() {
    with_regenerated_arenas::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal_multiple() {
    with_reused_arenas::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_once() {
    with_regenerated_arenas::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged_multiple() {
    with_reused_arenas::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >()
}
