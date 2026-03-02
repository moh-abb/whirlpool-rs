use crate::ast::pattern::Pattern;
use crate::ast::pattern::note::Frequency;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::examples::binary_tree_depth_two;
use crate::test::interpreter::examples::half_binary_tree_depth_two;
use crate::test::interpreter::examples::multiple_of_three_units;
use crate::test::interpreter::examples::multiple_of_unit_then_silence_then_unit;
use crate::test::interpreter::examples::one_cycle_silence;
use crate::test::interpreter::examples::one_cycle_unit;
use crate::test::interpreter::test_expectations;
use crate::test::interpreter::test_expectations_with_interpreter_setup;

#[test]
fn can_play_unit_for_one_cycle() {
    let note_unit = NoteUnit::Frequency(Frequency(440));
    // Test:
    // - Updating to one cycle will lead to exactly one invocation of
    // scheduling the note unit.
    // - Updating the cycle to the same time does nothing.
    let expectation = ScheduledExpectation {
        start_time: CycleTime::ZERO,
        duration: CycleTime::ONE,
        note_unit,
    };
    let expected_schedule_actions =
        [(CycleTime::ONE, &[expectation][..]), (CycleTime::ONE, &[][..])];
    let (arenas, head_index) = one_cycle_unit(note_unit);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_silence_for_one_cycle() {
    let expected_schedule_actions =
        [(CycleTime::from_int(1), &[][..]), (CycleTime::from_int(2), &[][..])];
    let (arenas, head_index) = one_cycle_silence();
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_double_alternating_cats() {
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let make_scheduled_action = |start_time, letter: Letter| {
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time),
            duration: CycleTime::ONE,
            note_unit: note_unit(letter),
        }]
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
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

fn play_double_cat_then_unit_with_multiplier(multiplier: CycleTime) {
    // [ Cat ] -----v
    //    ↓       ↓
    // [ Cat ]     C
    // ↓  ↓
    // A   B
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let unit_duration = multiplier.recip();
    let make_scheduled_action = |start_time, letter: Letter| {
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time).div(multiplier),
            duration: unit_duration,
            note_unit: note_unit(letter),
        }]
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
    play_double_cat_then_unit_with_multiplier(CycleTime::from_int(2))
}

#[test]
fn can_play_double_cat_then_unit_at_seven_times_speed() {
    play_double_cat_then_unit_with_multiplier(CycleTime::from_int(7))
}

fn play_cat_of_three_units_with_multiplier(multiplier: CycleTime) {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time, letter: Letter| {
        let result = [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time).div(multiplier),
            duration: multiplier.recip(),
            note_unit: note_unit(letter),
        }];
        println!("Result: {result:?}");
        result
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
    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset: CycleTime::ZERO, multiplier },
    );
}

#[test]
fn can_play_cat_of_three_units() {
    play_cat_of_three_units_with_multiplier(CycleTime::ONE);
}

#[test]
fn can_play_cat_of_three_units_at_double_speed() {
    play_cat_of_three_units_with_multiplier(CycleTime::from_int(2))
}

#[test]
fn can_play_cat_of_three_units_at_seven_times_speed() {
    play_cat_of_three_units_with_multiplier(CycleTime::from_int(7))
}

#[test]
fn can_play_cat_of_unit_then_silence_then_unit() {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  ~  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);

    let make_scheduled_action = |start_time, letter: Letter| {
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time),
            duration: CycleTime::ONE,
            note_unit: note_unit(letter),
        }]
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
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
