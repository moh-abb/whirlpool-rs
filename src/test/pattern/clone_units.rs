use core::mem;

use mockall::predicate;
use spin::Lazy;
use spin::Mutex;
use spin::rwlock::RwLock;

use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::equality::PatternOrdAdapter;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::test::pattern::mock::PatternArenaMockers;

type Slot<T> = Lazy<RwLock<Option<T>>>;
const fn empty_slot<T>() -> Slot<T> {
    Slot::new(|| RwLock::new(None))
}
const fn empty_slot_array<T, const N: usize>() -> [Slot<T>; N] {
    [const { empty_slot() }; N]
}
fn fill_slot<T>(slot: &Slot<T>, value: T) {
    let mut slot_inner = slot.write();
    assert!(slot_inner.is_none());
    let _ = slot_inner.insert(value);
}

fn test_clone_pattern_with_no_subpatterns(orig_pattern: Pattern) {
    static TEST_MUTEX: Mutex<()> = Mutex::new(());
    static PATTERN_SLOT: Slot<Pattern> = empty_slot();
    static CLONED_SLOT: Slot<Pattern> = empty_slot();
    // Acquire the mutex for unique access to the slots.
    TEST_MUTEX.lock();

    // Clear the slots.
    let _ = PATTERN_SLOT.write().take();
    let _ = CLONED_SLOT.write().take();

    // Set the initial pattern value.
    fill_slot(&PATTERN_SLOT, orig_pattern);

    let pattern_index = || Index::new(0x00);
    let cloned_index = || Index::new(0xC0);
    let mock_arenas = PatternArenaMockers::new();
    let mut pattern_mocker = mock_arenas.0.borrow_mut();
    pattern_mocker
        .expect_get_slot()
        .with(predicate::eq(pattern_index()))
        .returning(move |_| Ok(PATTERN_SLOT.read()));
    pattern_mocker
        .expect_alloc()
        .once()
        .with(predicate::always())
        .return_once(move |value| {
            let mut slot = CLONED_SLOT.write();
            assert!(slot.is_none());
            let _ = slot.insert(value);
            Ok(cloned_index())
        })
        .return_const(Ok(cloned_index()));
    pattern_mocker
        .expect_get_slot()
        .with(predicate::eq(cloned_index()))
        .returning(|_| Ok(CLONED_SLOT.read()));
    pattern_mocker
        .expect_get_mut_slot()
        .with(predicate::eq(cloned_index()))
        .returning(|_| Ok(CLONED_SLOT.write()));
    mem::drop(pattern_mocker);

    let mut orig_adapter =
        PatternCloneDropAdapter::new(pattern_index(), &mock_arenas);
    let _cloned_index = orig_adapter.clone().take_item();

    mock_arenas.0.borrow_mut().checkpoint();

    // Take the original adapter too, to avoid dropping it.
    let _pattern_index = orig_adapter.take_item();
}

#[test]
fn can_clone_silence() {
    test_clone_pattern_with_no_subpatterns(Pattern::Silence)
}

#[test]
fn can_clone_letter_a() {
    let pattern = Pattern::Note(NoteUnit::Letter(Letter::A));
    test_clone_pattern_with_no_subpatterns(pattern)
}

fn test_clone_pattern_with_three_subpatterns(
    multiple_to_pattern: impl FnOnce(Multiple<Pattern>) -> Pattern,
    is_expected_pattern: fn(&Pattern) -> bool,
) {
    static TEST_MUTEX: Mutex<()> = Mutex::new(());
    static PATTERN_SLOTS: [Slot<Pattern>; 10] = empty_slot_array();
    static PATTERN_CHAIN_SLOTS: [Slot<Chain<Pattern>>; 10] = empty_slot_array();
    // Acquire the mutex for unique access to the slots.
    TEST_MUTEX.lock();

    // Clear the slots.
    PATTERN_SLOTS.iter().for_each(|slot| {
        let _ = slot.write().take();
    });
    PATTERN_CHAIN_SLOTS
        .iter()
        .for_each(|slot| {
            let _ = slot.write().take();
        });

    fill_slot(
        &PATTERN_SLOTS[0],
        multiple_to_pattern(Multiple::new_nonempty(
            3,
            Index::new(0),
            Index::new(2),
        )),
    );
    let make_letter = |letter| Pattern::Note(NoteUnit::Letter(letter));
    let letter_a = || make_letter(Letter::A);
    let letter_b = || make_letter(Letter::B);
    let letter_c = || make_letter(Letter::C);
    fill_slot(&PATTERN_SLOTS[1], letter_a());
    fill_slot(&PATTERN_SLOTS[2], letter_b());
    fill_slot(&PATTERN_SLOTS[3], letter_c());
    let a_chain = Chain(Index::new(1), None, Some(Index::new(1)));
    let b_chain =
        Chain(Index::new(2), Some(Index::new(0)), Some(Index::new(2)));
    let c_chain = Chain(Index::new(3), Some(Index::new(1)), None);
    fill_slot(&PATTERN_CHAIN_SLOTS[0], a_chain);
    fill_slot(&PATTERN_CHAIN_SLOTS[1], b_chain);
    fill_slot(&PATTERN_CHAIN_SLOTS[2], c_chain);

    let mock_arenas = PatternArenaMockers::new();
    let mut pattern_mocker = mock_arenas.0.borrow_mut();
    let mut chain_mocker = mock_arenas.1.borrow_mut();
    pattern_mocker
        .expect_get_slot()
        .with(predicate::in_iter((0..=7).map(Index::new)))
        .returning(|index| Ok(PATTERN_SLOTS[usize::from(index)].read()));

    pattern_mocker
        .expect_alloc()
        .once()
        .with(predicate::function(is_expected_pattern))
        .return_once(move |value| {
            let mut slot = PATTERN_SLOTS[4].write();
            assert!(slot.is_none());
            let _ = slot.insert(value);
            Ok(Index::new(4))
        });
    let mut next_pattern_index = 5;
    pattern_mocker
        .expect_alloc()
        .times(3)
        .with(predicate::in_iter([letter_a(), letter_b(), letter_c()]))
        .returning(move |value| {
            let mut slot = PATTERN_SLOTS[next_pattern_index as usize].write();
            assert!(slot.is_none());
            let _ = slot.insert(value);
            next_pattern_index += 1;
            Ok(Index::new(next_pattern_index - 1))
        });
    pattern_mocker
        .expect_get_mut_slot()
        .with(predicate::eq(Index::new(4)))
        .returning(move |index| Ok(PATTERN_SLOTS[usize::from(index)].write()));

    let mut next_chain_index = 3;
    chain_mocker
        .expect_alloc()
        .times(3)
        .with(predicate::always())
        .returning(move |value| {
            let mut slot =
                PATTERN_CHAIN_SLOTS[next_chain_index as usize].write();
            assert!(slot.is_none());
            let _ = slot.insert(value);
            next_chain_index += 1;
            Ok(Index::new(next_chain_index - 1))
        });

    chain_mocker
        .expect_get_slot()
        .with(predicate::in_iter((0..6).map(Index::new)))
        .returning(|index| Ok(PATTERN_CHAIN_SLOTS[usize::from(index)].read()));
    chain_mocker
        .expect_get_mut_slot()
        .with(predicate::in_iter((3..6).map(Index::new)))
        .returning(|index| Ok(PATTERN_CHAIN_SLOTS[usize::from(index)].write()));
    chain_mocker
        .expect_get_mut_slot()
        .with(predicate::always())
        .returning(|index| panic!("Unexpected get mut at index: {index:?}"));
    mem::drop(chain_mocker);
    mem::drop(pattern_mocker);

    let pattern_index = Index::new(0);
    let mut orig_adapter =
        PatternCloneDropAdapter::new(pattern_index.clone(), &mock_arenas);
    let _cloned_index = orig_adapter.clone().take_item();
    assert_eq!(
        PatternOrdAdapter::new(pattern_index.clone(), &mock_arenas),
        PatternOrdAdapter::new(_cloned_index.clone(), &mock_arenas)
    );
    // Take the original adapter too, to avoid dropping them
    let _pattern_index = orig_adapter.take_item();

    mock_arenas.0.borrow_mut().checkpoint();
}

#[test]
fn can_clone_cat_with_length_three() {
    test_clone_pattern_with_three_subpatterns(Pattern::Cat, |p| {
        matches!(p, Pattern::Cat(_))
    })
}

#[test]
fn can_clone_seq_with_length_three() {
    test_clone_pattern_with_three_subpatterns(Pattern::Seq, |p| {
        matches!(p, Pattern::Seq(_))
    })
}

#[test]
fn can_clone_stack_with_length_three() {
    test_clone_pattern_with_three_subpatterns(Pattern::Stack, |p| {
        matches!(p, Pattern::Stack(_))
    })
}
