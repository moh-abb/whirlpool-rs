use core::fmt::Display;

use crate::ast::expr::ExprNode;
use crate::ast::expr::adapter::ExprArenaAdapter;
use crate::ast::expr::cmp::ExprOrdAdapter;
use crate::ast::expr::format::ExprDisplayAdapter;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::adapter::PatternArenaAdapter;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::arenas_test_run;
use crate::test::ast::arbitrary::strategy::AnyExpressionStrategy;
use crate::test::ast::arbitrary::strategy::AnyPatternStrategy;

struct DoNothing;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for DoNothing
{
    fn run<'r>(_: SharedArenaRef<'r, Arenas>, _: Index<PatternNode>) {}
}
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas> for DoNothing {
    fn run<'r>(_: SharedArenaRef<'r, Arenas>, _: Index<ExprNode>) {}
}

struct RunDropOnAdapter;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for RunDropOnAdapter
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        drop(PatternArenaAdapter::new(index, shared_arena_ref))
    }
}
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas>
    for RunDropOnAdapter
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<ExprNode>,
    ) {
        drop(ExprArenaAdapter::new(index, shared_arena_ref))
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
        drop(PatternArenaAdapter::new(index, shared_arena_ref.clone()));
        assert_eq!(shared_arena_ref.size(), Ok(0));
    }
}
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas>
    for DropPatternAndCheckArenasEmpty
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<ExprNode>,
    ) {
        assert_ne!(shared_arena_ref.size(), Ok(0));
        drop(ExprArenaAdapter::new(index, shared_arena_ref.clone()));
        assert_eq!(shared_arena_ref.size(), Ok(0));
    }
}

fn clone_and_test_equal<
    'r,
    'a: 'r,
    Node,
    Arenas: 'a,
    CloneAdapter: Clone,
    OrdAdapter: Ord,
    DisplayAdapter: Display,
>(
    shared_arena_ref: &'r SharedArenaRef<'a, Arenas>,
    index: Index<Node>,
    make_arena_adapter: impl Fn(
        Index<Node>,
        SharedArenaRef<'a, Arenas>,
    ) -> CloneAdapter,
    take_item: impl Fn(&mut CloneAdapter) -> ArenaResult<Option<Index<Node>>>,
    make_ord_adapter_left: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_ord_adapter_right: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_display_adapter: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> DisplayAdapter,
) {
    let mut adapter =
        make_arena_adapter(index.clone(), shared_arena_ref.clone());
    let cloned = take_item(&mut adapter.clone())
        .expect("Cloned adapter should not have arena error")
        .expect("Cloned adapter should have full index");
    // To avoid dropping the nodes, take the adapter's index.
    take_item(&mut adapter)
        .expect("Original adapter should not have arena error");

    let ord_left = make_ord_adapter_left(index.clone(), shared_arena_ref);
    let ord_right = make_ord_adapter_right(cloned.clone(), shared_arena_ref);
    let comparison = ord_left.cmp(&ord_right);

    assert!(
        comparison.is_eq(),
        "Pattern {} and cloned {} are distinct",
        make_display_adapter(index, shared_arena_ref),
        make_display_adapter(cloned, shared_arena_ref),
    )
}

struct CloneAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndCheckEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        clone_and_test_equal(
            &shared_arena_ref,
            index,
            PatternArenaAdapter::new,
            PatternArenaAdapter::take_index,
            PatternOrdAdapter::new_left,
            PatternOrdAdapter::new_right,
            PatternDisplayAdapter::new,
        );
    }
}
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas>
    for CloneAndCheckEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<ExprNode>,
    ) {
        clone_and_test_equal(
            &shared_arena_ref,
            index,
            ExprArenaAdapter::new,
            ExprArenaAdapter::take_index,
            ExprOrdAdapter::new_left,
            ExprOrdAdapter::new_right,
            ExprDisplayAdapter::new,
        );
    }
}

fn clone_and_drop_and_check_equal<
    'r,
    'a: 'r,
    Node,
    Arenas: 'a,
    CloneAdapter: Clone,
    OrdAdapter: Ord,
    DisplayAdapter: Display,
>(
    shared_arena_ref: &'r SharedArenaRef<'a, Arenas>,
    index: Index<Node>,
    make_arena_adapter: impl Fn(
        Index<Node>,
        SharedArenaRef<'a, Arenas>,
    ) -> CloneAdapter,
    take_item: impl Fn(&mut CloneAdapter) -> ArenaResult<Option<Index<Node>>>,
    make_ord_adapter_left: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_ord_adapter_right: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> OrdAdapter,
    make_display_adapter: impl Fn(
        Index<Node>,
        &'r SharedArenaRef<'a, Arenas>,
    ) -> DisplayAdapter,
) {
    let mut adapter =
        make_arena_adapter(index.clone(), shared_arena_ref.clone());
    let cloned = take_item(&mut adapter.clone())
        .expect("Cloned adapter should not have arena error")
        .expect("Cloned adapter should have full index");
    // To avoid dropping the nodes, take the adapter's index.
    take_item(&mut adapter)
        .expect("Original adapter should not have arena error");
    let cloned_adapter = make_arena_adapter(cloned, shared_arena_ref.clone());
    let cloned_2 = take_item(&mut cloned_adapter.clone())
        .expect("Cloned adapter 2 should not have arena error")
        .expect("Cloned adapter 2 should have full index");
    drop(cloned_adapter);
    let cloned_2_adapter =
        make_arena_adapter(cloned_2.clone(), shared_arena_ref.clone());
    let cloned_3 = take_item(&mut cloned_2_adapter.clone())
        .expect("Cloned adapter 3 should not have arena error")
        .expect("Cloned adapter 3 should have full index");
    drop(cloned_2_adapter);

    let ord_left = make_ord_adapter_left(cloned_3.clone(), &shared_arena_ref);
    let ord_right = make_ord_adapter_right(index.clone(), &shared_arena_ref);
    let comparison = ord_left.cmp(&ord_right);

    assert!(
        comparison.is_eq(),
        "Original linked structure {} and cloned {} are distinct",
        make_display_adapter(index, &shared_arena_ref),
        make_display_adapter(cloned_3, &shared_arena_ref),
    )
}

struct CloneAndDropAndCheckEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        clone_and_drop_and_check_equal(
            &shared_arena_ref,
            index,
            PatternArenaAdapter::new,
            PatternArenaAdapter::take_index,
            PatternOrdAdapter::new_left,
            PatternOrdAdapter::new_right,
            PatternDisplayAdapter::new,
        )
    }
}
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas>
    for CloneAndDropAndCheckEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<ExprNode>,
    ) {
        clone_and_drop_and_check_equal(
            &shared_arena_ref,
            index,
            ExprArenaAdapter::new,
            ExprArenaAdapter::take_index,
            ExprOrdAdapter::new_left,
            ExprOrdAdapter::new_right,
            ExprDisplayAdapter::new,
        )
    }
}

fn clone_and_drop_and_check_sizes_equal<
    'r,
    'a: 'r,
    Node,
    Arenas: 'a,
    CloneAdapter: Clone,
>(
    shared_arena_ref: &'r SharedArenaRef<'a, Arenas>,
    index: Index<Node>,
    make_arena_adapter: impl Fn(
        Index<Node>,
        SharedArenaRef<'a, Arenas>,
    ) -> CloneAdapter,
    take_item: impl Fn(&mut CloneAdapter) -> ArenaResult<Option<Index<Node>>>,
    get_size: impl Fn(&'r SharedArenaRef<'a, Arenas>) -> ArenaResult<usize>,
) {
    let sizes = get_size(shared_arena_ref).unwrap();
    let mut clone_adapter =
        make_arena_adapter(index.clone(), shared_arena_ref.clone());
    let index_2 = take_item(&mut clone_adapter.clone())
        .expect("Cloned adapter should not have arena error")
        .expect("Cloned adapter should have full index");
    let sizes_2 = get_size(shared_arena_ref).unwrap();
    let size_diff_1 = sizes_2 - sizes;
    let index_3 = take_item(&mut clone_adapter.clone())
        .expect("Cloned adapter 2 should not have arena error")
        .expect("Cloned adapter 2 should have full index");
    let sizes_3 = get_size(shared_arena_ref).unwrap();
    let size_diff_2 = sizes_3 - sizes_2;
    assert_eq!(
        size_diff_1, size_diff_2,
        "Cloning twice should increase the number of elements by the same amount"
    );
    let pattern_3_adapter =
        make_arena_adapter(index_3.clone(), shared_arena_ref.clone());
    drop(pattern_3_adapter);
    let sizes_4 = get_size(shared_arena_ref).unwrap();
    assert_eq!(
        sizes_4, sizes_2,
        "Cloning then dropping should preserve the number of elements"
    );
    drop(make_arena_adapter(index_2, shared_arena_ref.clone()));
    let sizes_5 = get_size(shared_arena_ref).unwrap();
    assert_eq!(
        sizes_5, sizes,
        "Cloning then dropping twice should preserve the number of elements"
    );
    // To check that the behaviour of the arena is preserved even while
    // the arena is filled with elements, we do not drop the provided
    // `pattern`.
    take_item(&mut clone_adapter)
        .expect("Cloned adapter should not have arena error")
        .expect("Cloned adapter should have full index");
}

struct CloneAndDropAndCheckSizesEqual;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for CloneAndDropAndCheckSizesEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<PatternNode>,
    ) {
        clone_and_drop_and_check_sizes_equal(
            &shared_arena_ref,
            index,
            PatternArenaAdapter::new,
            PatternArenaAdapter::take_index,
            Arena::size,
        );
    }
}
impl<Arenas: ExprArenas> ArenaTest<Index<ExprNode>, Arenas>
    for CloneAndDropAndCheckSizesEqual
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        index: Index<ExprNode>,
    ) {
        clone_and_drop_and_check_sizes_equal(
            &shared_arena_ref,
            index,
            ExprArenaAdapter::new,
            ExprArenaAdapter::take_index,
            Arena::size,
        );
    }
}

#[test]
fn can_allocate_pattern_in_growable_arenas() {
    arenas_test_run::<
        _,
        DoNothing,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_allocate_expression_in_growable_arenas() {
    arenas_test_run::<
        _,
        DoNothing,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArenaError,
    >()
}

#[test]
fn can_allocate_pattern_then_deallocate() {
    arenas_test_run::<
        _,
        RunDropOnAdapter,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_allocate_expression_then_deallocate() {
    arenas_test_run::<
        _,
        RunDropOnAdapter,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArenaError,
    >()
}

#[test]
fn can_allocate_pattern_then_deallocate_completely() {
    arenas_test_run::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_allocate_expression_then_deallocate_completely() {
    arenas_test_run::<
        _,
        DropPatternAndCheckArenasEmpty,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArenaError,
    >()
}

#[test]
fn can_clone_pattern_and_result_is_equal() {
    arenas_test_run::<
        _,
        CloneAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_clone_expression_and_result_is_equal() {
    arenas_test_run::<
        _,
        CloneAndCheckEqual,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArenaError,
    >()
}

#[test]
fn can_clone_and_drop_pattern_and_result_stays_equal() {
    arenas_test_run::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_clone_and_drop_expression_and_result_stays_equal() {
    arenas_test_run::<
        _,
        CloneAndDropAndCheckEqual,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArenaError,
    >()
}

#[test]
fn can_clone_and_drop_pattern_and_arena_sizes_are_unchanged() {
    arenas_test_run::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >()
}

#[test]
fn can_clone_and_drop_expression_and_arena_sizes_are_unchanged() {
    arenas_test_run::<
        _,
        CloneAndDropAndCheckSizesEqual,
        GrowableArena<ExprNode>,
        AnyExpressionStrategy,
        ArenaError,
    >()
}
