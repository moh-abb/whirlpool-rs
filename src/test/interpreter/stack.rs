use crate::ast::pattern::Pattern;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::examples::binary_tree_depth_two;
use crate::test::interpreter::examples::half_binary_tree_depth_two;
use crate::test::interpreter::examples::multiple_of_three_units;
use crate::test::interpreter::test_fixed_arenas;

#[test]
fn can_play_four_stacks() {
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let single_action = |start_time, letter: Letter| ScheduledExpectation {
        start_time: CycleTime::from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let make_scheduled_actions = |start_time| {
        [Letter::A, Letter::B, Letter::C, Letter::D]
            .map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_actions(0)[..]),
        (CycleTime::from_int(2), &make_scheduled_actions(1)[..]),
        (CycleTime::from_int(3), &make_scheduled_actions(2)[..]),
        (CycleTime::from_int(4), &make_scheduled_actions(3)[..]),
    ];
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Stack, Pattern::Stack, Pattern::Stack);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_double_stack_with_unit() {
    // [ Stack ] -----v
    //    ↓         ↓
    // [ Stack ]     C
    // ↓   ↓
    // A    B
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let single_action = |start_time, letter: Letter| ScheduledExpectation {
        start_time: CycleTime::from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let make_scheduled_actions = |start_time| {
        [Letter::A, Letter::B, Letter::C].map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_actions(0)[..]),
        (CycleTime::from_int(2), &make_scheduled_actions(1)[..]),
        (CycleTime::from_int(3), &make_scheduled_actions(2)[..]),
        (CycleTime::from_int(4), &make_scheduled_actions(3)[..]),
    ];
    let (arenas, head_index) =
        half_binary_tree_depth_two(Pattern::Stack, Pattern::Stack);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_stack_of_three_units() {
    // [ Stack ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let single_action = |start_time, letter: Letter| ScheduledExpectation {
        start_time: CycleTime::from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let make_scheduled_actions = |start_time| {
        [Letter::A, Letter::B, Letter::C].map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &make_scheduled_actions(0)[..]),
        (CycleTime::from_int(2), &make_scheduled_actions(1)[..]),
        (CycleTime::from_int(3), &make_scheduled_actions(2)[..]),
        (CycleTime::from_int(4), &make_scheduled_actions(3)[..]),
        (CycleTime::from_int(5), &make_scheduled_actions(4)[..]),
        (CycleTime::from_int(6), &make_scheduled_actions(5)[..]),
    ];
    let (arenas, head_index) = multiple_of_three_units(Pattern::Stack);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_stack_of_cats() {
    // [ stack ] ----v
    //    ↓        ↓
    // [ cat ]   [ cat ]
    // ↓  ↓    ↓  ↓
    // A   B     C   D
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let single_action = |start_time, letter: Letter| ScheduledExpectation {
        start_time: CycleTime::from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let even_cycle_actions = |start_time| {
        [Letter::A, Letter::C].map(|l| single_action(start_time, l))
    };
    let odd_cycle_actions = |start_time| {
        [Letter::B, Letter::D].map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::from_int(1), &even_cycle_actions(0)[..]),
        (CycleTime::from_int(2), &odd_cycle_actions(1)[..]),
        (CycleTime::from_int(3), &even_cycle_actions(2)[..]),
        (CycleTime::from_int(4), &odd_cycle_actions(3)[..]),
        (CycleTime::from_int(5), &even_cycle_actions(4)[..]),
        (CycleTime::from_int(6), &odd_cycle_actions(5)[..]),
    ];
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Stack, Pattern::Cat, Pattern::Cat);
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}
