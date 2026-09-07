use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::Arena;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::arenas_test_run;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;

struct DoNothing;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DoNothing
{
    fn run<'r>(_: SharedArenaRef<'r, Arenas>, _: Index<PatternNode>) {}
}

struct DropPattern;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DropPattern
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        drop(PatternDropAdapter::new(index, shared_arena_ref))
    }
}

struct DropPatternAndCheckArenasEmpty;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DropPatternAndCheckArenasEmpty
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        assert_ne!(shared_arena_ref.size(), Ok(0));
        drop(PatternDropAdapter::new(index, shared_arena_ref.clone()));
        assert_eq!(shared_arena_ref.size(), Ok(0));
    }
}

struct CloneAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndCheckEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        let mut adapter = PatternCloneDropAdapter::new(
            index.clone(),
            shared_arena_ref.clone(),
        );
        let cloned = adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        adapter.take_item();

        let ord_left =
            PatternOrdAdapter::new_left(index.clone(), &shared_arena_ref);
        let ord_right =
            PatternOrdAdapter::new_right(cloned.clone(), &shared_arena_ref);
        let comparison = ord_left.cmp(&ord_right);

        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(index, &shared_arena_ref),
            PatternDisplayAdapter::new(cloned, &shared_arena_ref),
        )
    }
}

struct CloneAndDropAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        let mut pattern_adapter = PatternCloneDropAdapter::new(
            index.clone(),
            shared_arena_ref.clone(),
        );
        let cloned = pattern_adapter.clone().take_item();
        // To avoid dropping the pattern, take the adapter's index.
        pattern_adapter.take_item();
        let cloned_adapter =
            PatternCloneDropAdapter::new(cloned, shared_arena_ref.clone());
        let cloned_2 = cloned_adapter.clone().take_item();
        drop(cloned_adapter);
        let cloned_2_adapter = PatternCloneDropAdapter::new(
            cloned_2.clone(),
            shared_arena_ref.clone(),
        );
        let cloned_3 = cloned_2_adapter.clone().take_item();
        drop(cloned_2_adapter);

        let ord_left =
            PatternOrdAdapter::new_left(cloned_3.clone(), &shared_arena_ref);
        let ord_right =
            PatternOrdAdapter::new_right(index.clone(), &shared_arena_ref);
        let comparison = ord_left.cmp(&ord_right);

        assert!(
            comparison.is_eq(),
            "Pattern {} and cloned {} are distinct",
            PatternDisplayAdapter::new(index, &shared_arena_ref),
            PatternDisplayAdapter::new(cloned_3, &shared_arena_ref),
        )
    }
}

struct CloneAndDropAndCheckSizesEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckSizesEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        let sizes = shared_arena_ref.size().unwrap();
        let mut clone_adapter = PatternCloneDropAdapter::new(
            index.clone(),
            shared_arena_ref.clone(),
        );
        let pattern_2 = clone_adapter.clone().take_item();
        let sizes_2 = shared_arena_ref.size().unwrap();
        let size_diff_1 = sizes_2 - sizes;
        let pattern_3 = clone_adapter.clone().take_item();
        let sizes_3 = shared_arena_ref.size().unwrap();
        let size_diff_2 = sizes_3 - sizes_2;
        assert_eq!(
            size_diff_1, size_diff_2,
            "Cloning twice should increase the number of elements by the same amount"
        );
        let pattern_3_adapter = PatternDropAdapter::new(
            pattern_3.clone(),
            shared_arena_ref.clone(),
        );
        drop(pattern_3_adapter);
        let sizes_4 = shared_arena_ref.size().unwrap();
        assert_eq!(
            sizes_4, sizes_2,
            "Cloning then dropping should preserve the number of elements"
        );
        drop(PatternDropAdapter::new(pattern_2, shared_arena_ref.clone()));
        let sizes_5 = shared_arena_ref.size().unwrap();
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
fn can_allocate_in_growable_arenas() {
    arenas_test_run::<
        _,
        DoNothing,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_allocate_then_deallocate() {
    arenas_test_run::<
        _,
        DropPattern,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_allocate_then_deallocate_completely() {
    arenas_test_run::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_clone_and_result_is_equal() {
    arenas_test_run::<
        _,
        CloneAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_clone_and_drop_many_times_and_result_stays_equal() {
    arenas_test_run::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_clone_and_drop_and_arena_sizes_are_unchanged() {
    arenas_test_run::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}
