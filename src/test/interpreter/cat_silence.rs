use crate::ast::pattern::Pattern;
use crate::ast::pattern::note::Frequency;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::test::interpreter::examples::binary_tree_depth_two;
use crate::test::interpreter::examples::half_binary_tree_depth_two;
use crate::test::interpreter::examples::multiple_of_three_units;
use crate::test::interpreter::examples::multiple_of_unit_then_silence_then_unit;
use crate::test::interpreter::examples::one_cycle_silence;
use crate::test::interpreter::examples::one_cycle_unit;
use crate::test::interpreter::test_fixed_arenas;

#[test]
fn can_play_unit_for_one_cycle() {
    let note_unit = NoteUnit::Frequency(Frequency(440));
    // Test:
    // - Updating to one cycle will lead to exactly one invocation of
    // scheduling the note unit.
    // - Updating the cycle to the same time does nothing.
    let expected_schedule_actions = [
        (CycleTime::ONE, &[(CycleTime::ZERO, CycleTime::ONE, note_unit)][..]),
        (CycleTime::ONE, &[][..]),
    ];
    let (arenas, head_index) = one_cycle_unit(note_unit);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_silence_for_one_cycle() {
    let expected_schedule_actions =
        [(CycleTime::from_int(1), &[][..]), (CycleTime::from_int(2), &[][..])];
    let (arenas, head_index) = one_cycle_silence();
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_double_alternating_cats() {
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let make_scheduled_action = |start_time: u32, letter: Letter| {
        [(CycleTime::from_int(start_time), CycleTime::ONE, note_unit(letter))]
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_action(0, Letter::A)[..]),
        (CycleTime::from_int(2), &make_scheduled_action(1, Letter::C)[..]),
        (CycleTime::from_int(3), &make_scheduled_action(2, Letter::B)[..]),
        (CycleTime::from_int(4), &make_scheduled_action(3, Letter::D)[..]),
        (CycleTime::from_int(5), &make_scheduled_action(4, Letter::A)[..]),
        (CycleTime::from_int(6), &make_scheduled_action(5, Letter::C)[..]),
        (CycleTime::from_int(7), &make_scheduled_action(6, Letter::B)[..]),
        (CycleTime::from_int(8), &make_scheduled_action(7, Letter::D)[..]),
    ];
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Cat, Pattern::Cat, Pattern::Cat);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_double_cat_then_unit() {
    // [ Cat ] -----v
    //    ↓       ↓
    // [ Cat ]     C
    // ↓  ↓
    // A   B
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time: u32, letter: Letter| {
        [(CycleTime::from_int(start_time), CycleTime::ONE, note_unit(letter))]
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_action(0, Letter::A)[..]),
        (CycleTime::from_int(2), &make_scheduled_action(1, Letter::C)[..]),
        (CycleTime::from_int(3), &make_scheduled_action(2, Letter::B)[..]),
        (CycleTime::from_int(4), &make_scheduled_action(3, Letter::C)[..]),
        (CycleTime::from_int(5), &make_scheduled_action(4, Letter::A)[..]),
        (CycleTime::from_int(6), &make_scheduled_action(5, Letter::C)[..]),
        (CycleTime::from_int(7), &make_scheduled_action(6, Letter::B)[..]),
        (CycleTime::from_int(8), &make_scheduled_action(7, Letter::C)[..]),
    ];
    let (arenas, head_index) =
        half_binary_tree_depth_two(Pattern::Cat, Pattern::Cat);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_cat_of_three_units() {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time: u32, letter: Letter| {
        [(CycleTime::from_int(start_time), CycleTime::ONE, note_unit(letter))]
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_action(0, Letter::A)[..]),
        (CycleTime::from_int(2), &make_scheduled_action(1, Letter::B)[..]),
        (CycleTime::from_int(3), &make_scheduled_action(2, Letter::C)[..]),
        (CycleTime::from_int(4), &make_scheduled_action(3, Letter::A)[..]),
        (CycleTime::from_int(5), &make_scheduled_action(4, Letter::B)[..]),
        (CycleTime::from_int(6), &make_scheduled_action(5, Letter::C)[..]),
    ];
    let (arenas, head_index) = multiple_of_three_units(Pattern::Cat);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_cat_of_unit_then_silence_then_unit() {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  ~  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time: u32, letter: Letter| {
        [(CycleTime::from_int(start_time), CycleTime::ONE, note_unit(letter))]
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_action(0, Letter::A)[..]),
        (CycleTime::from_int(2), &[][..]),
        (CycleTime::from_int(3), &make_scheduled_action(2, Letter::C)[..]),
        (CycleTime::from_int(4), &make_scheduled_action(3, Letter::A)[..]),
        (CycleTime::from_int(5), &[][..]),
        (CycleTime::from_int(6), &make_scheduled_action(5, Letter::C)[..]),
    ];
    let (arenas, head_index) =
        multiple_of_unit_then_silence_then_unit(Pattern::Cat);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}
