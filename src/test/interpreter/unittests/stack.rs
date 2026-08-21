use crate::ast::CycleTime;
use crate::ast::Note;
use crate::ast::NoteLetter;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::test::examples::unittests::pattern::binary_tree_depth_two;
use crate::test::examples::unittests::pattern::half_binary_tree_depth_two;
use crate::test::examples::unittests::pattern::multiple_of_three_units;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::test_expectations;

#[test]
fn can_play_four_stacks() {
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));
    let single_action = |start_time, letter: NoteLetter| ScheduledExpectation {
        start_time: CycleTime::unwrapped_from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let make_scheduled_actions = |start_time| {
        [NoteLetter::A, NoteLetter::B, NoteLetter::C, NoteLetter::D]
            .map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::unwrapped_from_int(1), &make_scheduled_actions(0)[..]),
        (CycleTime::unwrapped_from_int(2), &make_scheduled_actions(1)[..]),
        (CycleTime::unwrapped_from_int(3), &make_scheduled_actions(2)[..]),
        (CycleTime::unwrapped_from_int(4), &make_scheduled_actions(3)[..]),
    ];
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Stack, Pattern::Stack, Pattern::Stack);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_double_stack_with_unit() {
    // [ Stack ] -----v
    //    ↓         ↓
    // [ Stack ]     C
    // ↓   ↓
    // A    B
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));

    let single_action = |start_time, letter: NoteLetter| ScheduledExpectation {
        start_time: CycleTime::unwrapped_from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let make_scheduled_actions = |start_time| {
        [NoteLetter::A, NoteLetter::B, NoteLetter::C]
            .map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::unwrapped_from_int(1), &make_scheduled_actions(0)[..]),
        (CycleTime::unwrapped_from_int(2), &make_scheduled_actions(1)[..]),
        (CycleTime::unwrapped_from_int(3), &make_scheduled_actions(2)[..]),
        (CycleTime::unwrapped_from_int(4), &make_scheduled_actions(3)[..]),
    ];
    let (arenas, head_index) =
        half_binary_tree_depth_two(Pattern::Stack, Pattern::Stack);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_stack_of_three_units() {
    // [ Stack ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));

    let single_action = |start_time, letter: NoteLetter| ScheduledExpectation {
        start_time: CycleTime::unwrapped_from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let make_scheduled_actions = |start_time| {
        [NoteLetter::A, NoteLetter::B, NoteLetter::C]
            .map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::unwrapped_from_int(1), &make_scheduled_actions(0)[..]),
        (CycleTime::unwrapped_from_int(2), &make_scheduled_actions(1)[..]),
        (CycleTime::unwrapped_from_int(3), &make_scheduled_actions(2)[..]),
        (CycleTime::unwrapped_from_int(4), &make_scheduled_actions(3)[..]),
        (CycleTime::unwrapped_from_int(5), &make_scheduled_actions(4)[..]),
        (CycleTime::unwrapped_from_int(6), &make_scheduled_actions(5)[..]),
    ];
    let (arenas, head_index) = multiple_of_three_units(Pattern::Stack);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_stack_of_cats() {
    // [ stack ] ----v
    //    ↓        ↓
    // [ cat ]   [ cat ]
    // ↓  ↓    ↓  ↓
    // A   B     C   D
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));

    let single_action = |start_time, letter: NoteLetter| ScheduledExpectation {
        start_time: CycleTime::unwrapped_from_int(start_time),
        duration: CycleTime::ONE,
        note_unit: note_unit(letter),
    };
    let even_cycle_actions = |start_time| {
        [NoteLetter::A, NoteLetter::C].map(|l| single_action(start_time, l))
    };
    let odd_cycle_actions = |start_time| {
        [NoteLetter::B, NoteLetter::D].map(|l| single_action(start_time, l))
    };
    let expected_schedule_actions = [
        (CycleTime::unwrapped_from_int(1), &even_cycle_actions(0)[..]),
        (CycleTime::unwrapped_from_int(2), &odd_cycle_actions(1)[..]),
        (CycleTime::unwrapped_from_int(3), &even_cycle_actions(2)[..]),
        (CycleTime::unwrapped_from_int(4), &odd_cycle_actions(3)[..]),
        (CycleTime::unwrapped_from_int(5), &even_cycle_actions(4)[..]),
        (CycleTime::unwrapped_from_int(6), &odd_cycle_actions(5)[..]),
    ];
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Stack, Pattern::Cat, Pattern::Cat);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
