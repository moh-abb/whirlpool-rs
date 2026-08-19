use crate::ast::CycleTime;
use crate::ast::NoteLetter;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::test::examples::unittests::pattern::binary_tree_depth_two;
use crate::test::examples::unittests::pattern::half_binary_tree_depth_two;
use crate::test::examples::unittests::pattern::multiple_of_three_units;
use crate::test::examples::unittests::pattern::multiple_of_unit_then_silence_then_unit;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::test_expectations;
use crate::test::interpreter::test_expectations_with_interpreter_setup;
use crate::test::interpreter::unittests::NONZERO_OFFSET;

fn play_double_alternating_cats_with_offset_and_multiplier(
    offset: CycleTime,
    multiplier: CycleTime,
) {
    let note_unit = |letter: NoteLetter| NoteUnit::Letter(letter);
    let unit_duration = multiplier.recip().unwrap();
    let make_scheduled_action = |start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time: (CycleTime::unwrapped_from_int(start_time) + offset)
                / multiplier,
            duration: unit_duration,
            note_unit: note_unit(letter),
        }]
    };
    let times_and_actions = [
        &(1, make_scheduled_action(0, NoteLetter::A)),
        &(2, make_scheduled_action(1, NoteLetter::C)),
        &(3, make_scheduled_action(2, NoteLetter::B)),
        &(4, make_scheduled_action(3, NoteLetter::D)),
        &(5, make_scheduled_action(4, NoteLetter::A)),
        &(6, make_scheduled_action(5, NoteLetter::C)),
        &(7, make_scheduled_action(6, NoteLetter::B)),
        &(8, make_scheduled_action(7, NoteLetter::D)),
    ];
    let expected_schedule_actions = times_and_actions.map(|(time, actions)| {
        (CycleTime::unwrapped_from_int(*time), &actions[..])
    });
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Cat, Pattern::Cat, Pattern::Cat);
    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset, multiplier },
    );
}

#[test]
fn can_play_double_alternating_cats() {
    play_double_alternating_cats_with_offset_and_multiplier(
        CycleTime::ZERO,
        CycleTime::ONE,
    );
}

#[test]
fn can_play_double_alternating_cats_at_double_speed() {
    play_double_alternating_cats_with_offset_and_multiplier(
        CycleTime::ZERO,
        CycleTime::unwrapped_from_int(2),
    );
}

#[test]
fn can_play_double_alternating_cats_at_seven_times_speed() {
    play_double_alternating_cats_with_offset_and_multiplier(
        CycleTime::ZERO,
        CycleTime::unwrapped_from_int(7),
    );
}

#[test]
fn can_play_double_alternating_cats_at_nonzero_offset() {
    play_double_alternating_cats_with_offset_and_multiplier(
        NONZERO_OFFSET,
        CycleTime::ONE,
    );
}

#[test]
fn can_play_double_alternating_cats_at_double_speed_and_nonzero_offset() {
    play_double_alternating_cats_with_offset_and_multiplier(
        NONZERO_OFFSET,
        CycleTime::unwrapped_from_int(2),
    );
}

#[test]
fn can_play_double_alternating_cats_at_seven_times_speed_and_nonzero_offset() {
    play_double_alternating_cats_with_offset_and_multiplier(
        NONZERO_OFFSET,
        CycleTime::unwrapped_from_int(7),
    );
}

fn play_double_cat_then_unit_with_multiplier(multiplier: CycleTime) {
    // [ Cat ] -----v
    //    ↓       ↓
    // [ Cat ]     C
    // ↓  ↓
    // A   B
    let note_unit = |letter: NoteLetter| NoteUnit::Letter(letter);

    let unit_duration = multiplier.recip().unwrap();
    let make_scheduled_action = |start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time: CycleTime::unwrapped_from_int(start_time) / multiplier,
            duration: unit_duration,
            note_unit: note_unit(letter),
        }]
    };
    let times_and_actions = [
        &(1, make_scheduled_action(0, NoteLetter::A)),
        &(2, make_scheduled_action(1, NoteLetter::C)),
        &(3, make_scheduled_action(2, NoteLetter::B)),
        &(4, make_scheduled_action(3, NoteLetter::C)),
        &(5, make_scheduled_action(4, NoteLetter::A)),
        &(6, make_scheduled_action(5, NoteLetter::C)),
        &(7, make_scheduled_action(6, NoteLetter::B)),
        &(8, make_scheduled_action(7, NoteLetter::C)),
    ];
    let expected_schedule_actions = times_and_actions.map(|(time, actions)| {
        (CycleTime::unwrapped_from_int(*time), &actions[..])
    });
    let (arenas, head_index) =
        half_binary_tree_depth_two(Pattern::Cat, Pattern::Cat);
    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset: CycleTime::ZERO, multiplier },
    );
}

#[test]
fn can_play_double_cat_then_unit() {
    play_double_cat_then_unit_with_multiplier(CycleTime::ONE)
}

#[test]
fn can_play_double_cat_then_unit_at_double_speed() {
    play_double_cat_then_unit_with_multiplier(CycleTime::unwrapped_from_int(2))
}

#[test]
fn can_play_double_cat_then_unit_at_seven_times_speed() {
    play_double_cat_then_unit_with_multiplier(CycleTime::unwrapped_from_int(7))
}

fn play_cat_of_three_units_with_offset_and_multiplier(
    offset: CycleTime,
    multiplier: CycleTime,
) {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: NoteLetter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time: (CycleTime::unwrapped_from_int(start_time) + offset)
                / multiplier,
            duration: multiplier.recip().unwrap(),
            note_unit: note_unit(letter),
        }]
    };
    let times_and_actions = [
        &(1, make_scheduled_action(0, NoteLetter::A)),
        &(2, make_scheduled_action(1, NoteLetter::B)),
        &(3, make_scheduled_action(2, NoteLetter::C)),
        &(4, make_scheduled_action(3, NoteLetter::A)),
        &(5, make_scheduled_action(4, NoteLetter::B)),
        &(6, make_scheduled_action(5, NoteLetter::C)),
    ];
    let expected_schedule_actions = times_and_actions.map(|(time, actions)| {
        (CycleTime::unwrapped_from_int(*time), &actions[..])
    });
    let (arenas, head_index) = multiple_of_three_units(Pattern::Cat);
    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset, multiplier },
    );
}

#[test]
fn can_play_cat_of_three_units() {
    play_cat_of_three_units_with_offset_and_multiplier(
        CycleTime::ZERO,
        CycleTime::ONE,
    );
}

#[test]
fn can_play_cat_of_three_units_at_double_speed() {
    play_cat_of_three_units_with_offset_and_multiplier(
        CycleTime::ZERO,
        CycleTime::unwrapped_from_int(2),
    )
}

#[test]
fn can_play_cat_of_three_units_at_seven_times_speed() {
    play_cat_of_three_units_with_offset_and_multiplier(
        CycleTime::ZERO,
        CycleTime::unwrapped_from_int(7),
    )
}

#[test]
fn can_play_cat_of_three_units_at_nonzero_offset() {
    play_cat_of_three_units_with_offset_and_multiplier(
        NONZERO_OFFSET,
        CycleTime::ONE,
    );
}

#[test]
fn can_play_cat_of_three_units_at_double_speed_and_nonzero_offset() {
    play_cat_of_three_units_with_offset_and_multiplier(
        NONZERO_OFFSET,
        CycleTime::unwrapped_from_int(2),
    )
}

#[test]
fn can_play_cat_of_three_units_at_seven_times_speed_and_nonzero_offset() {
    play_cat_of_three_units_with_offset_and_multiplier(
        NONZERO_OFFSET,
        CycleTime::unwrapped_from_int(7),
    )
}

#[test]
fn can_play_cat_of_unit_then_silence_then_unit() {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  ~  C
    let note_unit = |letter: NoteLetter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time: CycleTime::unwrapped_from_int(start_time),
            duration: CycleTime::ONE,
            note_unit: note_unit(letter),
        }]
    };
    let times_and_actions = [
        &(1, make_scheduled_action(0, NoteLetter::A)),
        &(3, make_scheduled_action(2, NoteLetter::C)),
        &(4, make_scheduled_action(3, NoteLetter::A)),
        &(6, make_scheduled_action(5, NoteLetter::C)),
    ];
    let expected_schedule_actions = times_and_actions.map(|(time, actions)| {
        (CycleTime::unwrapped_from_int(*time), &actions[..])
    });
    let (arenas, head_index) =
        multiple_of_unit_then_silence_then_unit(Pattern::Cat);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
