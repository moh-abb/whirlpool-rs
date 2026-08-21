use alloc::rc::Rc;
use core::cell::RefCell;

use mockall::predicate;

use crate::ast::CycleTime;
use crate::ast::Note;
use crate::ast::NoteLetter;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::mem::Arena;
use crate::mem::ArenaMocker;
use crate::mem::Chain;
use crate::mem::Cow;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::arena::arena_impl::shared_arena::SharedArena;

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
    let mut mock_arenas = ArenaMocker::<PatternNode>::new();
    let mut pattern_mocker = mock_arenas.borrow_mut();

    pattern_mocker
        .expect_get_slot()
        .with(predicate::eq(pattern_index()))
        .returning_st(move |_| Ok(pattern_slot.clone()));
    let first_cloned_slot = cloned_slot.clone();
    pattern_mocker
        .expect_push()
        .once()
        .with(predicate::always())
        .returning_st(move |value| {
            fill_slot(&first_cloned_slot, value);
            Ok(cloned_index())
        });
    let second_cloned_slot = cloned_slot.clone();
    let third_cloned_slot = cloned_slot.clone();
    pattern_mocker
        .expect_get_slot()
        .with(predicate::eq(cloned_index()))
        .returning_st(move |_| Ok(second_cloned_slot.clone()));
    pattern_mocker
        .expect_get_mut_slot()
        .with(predicate::eq(cloned_index()))
        .returning_st(move |_| Ok(third_cloned_slot.clone()));
    drop(pattern_mocker);

    let shared_mock_arenas = SharedArena::new(&mut mock_arenas);

    let mut orig_adapter = PatternCloneDropAdapter::new(
        pattern_index(),
        shared_mock_arenas.make_ref(),
    );
    let _cloned_index = orig_adapter.clone().take_item();

    assert_eq!(_cloned_index, cloned_index());
    assert_eq!(
        PatternOrdAdapter::new_left(
            pattern_index(),
            &shared_mock_arenas.make_ref()
        ),
        PatternOrdAdapter::new_right(
            cloned_index(),
            &shared_mock_arenas.make_ref()
        )
    );
    // Take the original adapter too, to avoid dropping it.
    let _pattern_index = orig_adapter.take_item();
    assert_eq!(_pattern_index, pattern_index());
    drop(orig_adapter);
    drop(shared_mock_arenas);
    mock_arenas.borrow_mut().checkpoint();
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
    let pattern = Pattern::Note(NoteUnit::WithOctave(Note::new(NoteLetter::A)));
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
            pattern: Pattern::Note(NoteUnit::WithOctave(Note::new(letter))),
        })
    };

    let mut arenas = GrowableArena::<PatternNode>::default();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

    // Link the elements to the parent.
    // Try: [] becomes [B] becomes [A, B] becomes [A, B, C].
    shared_arena_ref_1
        .push(PatternNode {
            parent: None,
            sibling_chain: Chain::new(),
            pattern: multiple_to_pattern(Multiple::new()),
        })
        .unwrap();
    Multiple::push_back(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        parent_index.clone(),
        make_letter(NoteLetter::B),
    )
    .unwrap();
    Multiple::push_front(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        parent_index.clone(),
        make_letter(NoteLetter::A),
    )
    .unwrap();
    Multiple::push_back(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        parent_index.clone(),
        make_letter(NoteLetter::C),
    )
    .unwrap();

    let mut orig_adapter =
        PatternCloneDropAdapter::new(parent_index.clone(), shared_arena_ref_1);
    let cloned_index = orig_adapter.clone().take_item();
    assert_ne!(parent_index, cloned_index);
    assert_eq!(
        PatternOrdAdapter::new_left(parent_index.clone(), &shared_arena_ref_1),
        PatternOrdAdapter::new_right(cloned_index.clone(), &shared_arena_ref_1)
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
