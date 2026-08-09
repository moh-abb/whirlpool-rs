use alloc::rc::Rc;
use core::cell::RefCell;
use core::mem;

use mockall::predicate;

use crate::ast::CycleTime;
use crate::ast::NoteLetter;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::mem::Arena;
use crate::mem::Chain;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::test::pattern::arenas::GrowableArenas;
use crate::test::pattern::arenas::PatternArenaMocker;

type Slot<T> = Rc<RefCell<Option<T>>>;
fn empty_slot<T>() -> Slot<T> {
    Rc::new(RefCell::new(None))
}
fn fill_slot<T>(slot: &Slot<T>, value: T) {
    let mut slot_inner = slot.borrow_mut();
    assert!(slot_inner.is_none());
    let _ = slot_inner.insert(value);
}

fn test_clone_with_no_subpatterns(orig_pattern: Pattern) {
    // Start with empty slots.
    let pattern_slot: Slot<PatternNode> = empty_slot();
    let cloned_slot: Slot<PatternNode> = empty_slot();

    // Set the initial pattern value.
    fill_slot(
        &pattern_slot,
        PatternNode {
            parent: None,
            sibling_chain: Chain::new(),
            pattern: orig_pattern,
        },
    );

    let pattern_index = || Index::new(0x00);
    let cloned_index = || Index::new(0xC0);
    let mock_arenas = PatternArenaMocker::new();
    let mut pattern_mocker = mock_arenas.0.borrow_mut();
    pattern_mocker
        .expect_get_slot()
        .with(predicate::eq(pattern_index()))
        .returning_st(move |_| Ok(pattern_slot.clone()));
    let first_cloned_slot = cloned_slot.clone();
    pattern_mocker
        .expect_push()
        .once()
        .with(predicate::always())
        .return_once_st(move |value| {
            fill_slot(&first_cloned_slot, value);
            Ok(cloned_index())
        });
    let second_cloned_slot = cloned_slot.clone();
    let third_cloned_slot = cloned_slot.clone();
    pattern_mocker
        .expect_get_slot()
        .with(predicate::eq(cloned_index()))
        .return_once_st(move |_| Ok(second_cloned_slot));
    pattern_mocker
        .expect_get_mut_slot()
        .with(predicate::eq(cloned_index()))
        .return_once_st(move |_| Ok(third_cloned_slot));
    mem::drop(pattern_mocker);

    let mut orig_adapter =
        PatternCloneDropAdapter::new(pattern_index(), &mock_arenas);
    let _cloned_index = orig_adapter.clone().take_item();

    assert_eq!(_cloned_index, cloned_index());
    assert_eq!(
        PatternOrdAdapter::new_left(pattern_index(), &mock_arenas),
        PatternOrdAdapter::new_right(cloned_index(), &mock_arenas)
    );
    // Take the original adapter too, to avoid dropping it.
    let _pattern_index = orig_adapter.take_item();
    assert_eq!(_pattern_index, pattern_index());

    mock_arenas.0.borrow_mut().checkpoint();
}

#[test]
fn can_clone_silence() {
    test_clone_with_no_subpatterns(Pattern::Silence)
}

#[test]
fn can_clone_empty_cat() {
    test_clone_with_no_subpatterns(Pattern::Cat(Multiple::new()))
}

#[test]
fn can_clone_empty_seq() {
    test_clone_with_no_subpatterns(Pattern::Seq(Multiple::new()))
}

#[test]
fn can_clone_empty_stack() {
    test_clone_with_no_subpatterns(Pattern::Stack(Multiple::new()))
}

#[test]
fn can_clone_empty_time_cat() {
    test_clone_with_no_subpatterns(Pattern::TimeCat {
        total_cycle_length: CycleTime::ZERO,
        multiple: Multiple::new(),
    });
}

#[test]
fn can_clone_empty_arrange() {
    test_clone_with_no_subpatterns(Pattern::Arrange {
        total_cycle_length: CycleTime::ZERO,
        multiple: Multiple::new(),
    });
}

#[test]
fn can_clone_letter_a() {
    let pattern = Pattern::Note(NoteUnit::Letter(NoteLetter::A));
    test_clone_with_no_subpatterns(pattern)
}

fn test_clone_with_three_subpatterns(
    multiple_to_pattern: impl FnOnce(Multiple<PatternNode>) -> Pattern,
) {
    let parent_index = Index::new(0);

    let make_letter = |letter| {
        Cow::Owned(PatternNode {
            parent: None,
            sibling_chain: Chain::new(),
            pattern: Pattern::Note(NoteUnit::Letter(letter)),
        })
    };

    let arenas = GrowableArenas::default();

    // Link the elements to the parent.
    // Try: [] becomes [B] becomes [A, B] becomes [A, B, C].
    arenas
        .get_pattern_arena()
        .push(PatternNode {
            parent: None,
            sibling_chain: Chain::new(),
            pattern: multiple_to_pattern(Multiple::new()),
        })
        .unwrap();
    Multiple::push_back(
        &arenas,
        parent_index.clone(),
        make_letter(NoteLetter::B),
    )
    .unwrap();
    Multiple::push_front(
        &arenas,
        parent_index.clone(),
        make_letter(NoteLetter::A),
    )
    .unwrap();
    Multiple::push_back(
        &arenas,
        parent_index.clone(),
        make_letter(NoteLetter::C),
    )
    .unwrap();

    let mut orig_adapter =
        PatternCloneDropAdapter::new(parent_index.clone(), &arenas);
    let cloned_index = orig_adapter.clone().take_item();
    assert_ne!(parent_index, cloned_index);
    assert_eq!(
        PatternOrdAdapter::new_left(parent_index.clone(), &arenas),
        PatternOrdAdapter::new_right(cloned_index.clone(), &arenas)
    );
    // Take the original adapter too, to avoid dropping them
    let _parent_index = orig_adapter.take_item();
}

#[test]
fn can_clone_cat_with_length_three() {
    test_clone_with_three_subpatterns(Pattern::Cat)
}

#[test]
fn can_clone_seq_with_length_three() {
    test_clone_with_three_subpatterns(Pattern::Seq)
}

#[test]
fn can_clone_stack_with_length_three() {
    test_clone_with_three_subpatterns(Pattern::Stack)
}
