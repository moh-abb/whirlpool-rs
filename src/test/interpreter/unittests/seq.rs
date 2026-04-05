use crate::ast::pattern::Pattern;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::test_expectations;
use crate::test::interpreter::test_expectations_with_interpreter_setup;
use crate::test::interpreter::unittests::examples::binary_tree_depth_two;
use crate::test::interpreter::unittests::examples::half_binary_tree_depth_two;
use crate::test::interpreter::unittests::examples::multiple_of_three_units;
use crate::test::interpreter::unittests::examples::multiple_of_unit_then_silence_then_unit;

fn play_double_sequential_seqs_with_multiplier(multiplier: CycleTime) {
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let multiple_length_product = CycleTime::from_int(4);

    let interpreter_time =
        |start_time| CycleTime::from_int(start_time) / multiple_length_product;
    let make_scheduled_action = |start_time, letter: Letter| {
        let unit_duration = (multiple_length_product * multiplier).recip();
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time)
                / (multiple_length_product * multiplier),
            duration: unit_duration,
            note_unit: note_unit(letter),
        }]
    };
    let expected_schedule_actions = [
        (interpreter_time(1), &make_scheduled_action(0, Letter::A)[..]),
        (interpreter_time(2), &make_scheduled_action(1, Letter::B)[..]),
        (interpreter_time(3), &make_scheduled_action(2, Letter::C)[..]),
        (interpreter_time(4), &make_scheduled_action(3, Letter::D)[..]),
        (interpreter_time(5), &make_scheduled_action(4, Letter::A)[..]),
        (interpreter_time(6), &make_scheduled_action(5, Letter::B)[..]),
        (interpreter_time(7), &make_scheduled_action(6, Letter::C)[..]),
        (interpreter_time(8), &make_scheduled_action(7, Letter::D)[..]),
    ];
    let (arenas, head_index) =
        binary_tree_depth_two(Pattern::Seq, Pattern::Seq, Pattern::Seq);

    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset: CycleTime::ZERO, multiplier },
    );
}

#[test]
fn can_play_double_sequential_seqs() {
    play_double_sequential_seqs_with_multiplier(CycleTime::ONE);
}

#[test]
fn can_play_double_sequential_seqs_at_double_speed() {
    play_double_sequential_seqs_with_multiplier(CycleTime::from_int(2));
}

#[test]
fn can_play_double_sequential_seqs_at_seven_times_speed() {
    play_double_sequential_seqs_with_multiplier(CycleTime::from_int(7));
}

fn play_double_seq_then_unit_with_multiplier(multiplier: CycleTime) {
    // [ Seq ] -----v
    //    ↓       ↓
    // [ Seq ]     C
    // ↓  ↓
    // A   B
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let multiple_length_product = CycleTime::from_int(4);

    let interpreter_time =
        |start_time| CycleTime::from_int(start_time) / multiple_length_product;
    let make_scheduled_action = |start_time, duration_recip, letter: Letter| {
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time)
                / (multiplier * multiple_length_product),
            duration: CycleTime::from_int_recip(duration_recip).div(multiplier),
            note_unit: note_unit(letter),
        }]
    };
    let expected_schedule_actions = [
        (interpreter_time(1), &make_scheduled_action(0, 4, Letter::A)[..]),
        (interpreter_time(2), &make_scheduled_action(1, 4, Letter::B)[..]),
        (interpreter_time(3), &make_scheduled_action(2, 2, Letter::C)[..]),
        (interpreter_time(4), &[][..]),
        (interpreter_time(5), &make_scheduled_action(4, 4, Letter::A)[..]),
        (interpreter_time(6), &make_scheduled_action(5, 4, Letter::B)[..]),
        (interpreter_time(7), &make_scheduled_action(6, 2, Letter::C)[..]),
        (interpreter_time(8), &[][..]),
    ];
    let (arenas, head_index) =
        half_binary_tree_depth_two(Pattern::Seq, Pattern::Seq);
    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset: CycleTime::ZERO, multiplier },
    );
}

#[test]
fn can_play_double_seq_then_unit() {
    play_double_seq_then_unit_with_multiplier(CycleTime::ONE)
}

#[test]
fn can_play_double_seq_then_unit_at_double_speed() {
    play_double_seq_then_unit_with_multiplier(CycleTime::from_int(2))
}

#[test]
fn can_play_double_seq_then_unit_at_seven_times_speed() {
    play_double_seq_then_unit_with_multiplier(CycleTime::from_int(7))
}

fn play_seq_of_three_units_with_multiplier(multiplier: CycleTime) {
    // [  Seq  ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let multiple_length = CycleTime::from_int(3);

    let make_scheduled_action = |start_time, letter: Letter| {
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time)
                / (multiplier * multiple_length),
            duration: multiplier.mul(multiple_length).recip(),
            note_unit: note_unit(letter),
        }]
    };
    let interpreter_time =
        |start_time| CycleTime::from_int(start_time) / multiple_length;
    let expected_schedule_actions = [
        (interpreter_time(1), &make_scheduled_action(0, Letter::A)[..]),
        (interpreter_time(2), &make_scheduled_action(1, Letter::B)[..]),
        (interpreter_time(3), &make_scheduled_action(2, Letter::C)[..]),
        (interpreter_time(4), &make_scheduled_action(3, Letter::A)[..]),
        (interpreter_time(5), &make_scheduled_action(4, Letter::B)[..]),
        (interpreter_time(6), &make_scheduled_action(5, Letter::C)[..]),
    ];
    println!("Expectations: {expected_schedule_actions:?}");
    let (arenas, head_index) = multiple_of_three_units(Pattern::Seq);
    test_expectations_with_interpreter_setup(
        &arenas,
        head_index,
        &expected_schedule_actions,
        FullInterpreterSetup { offset: CycleTime::ZERO, multiplier },
    );
}

#[test]
fn can_play_seq_of_three_units() {
    play_seq_of_three_units_with_multiplier(CycleTime::ONE);
}

#[test]
fn can_play_seq_of_three_units_at_double_speed() {
    play_seq_of_three_units_with_multiplier(CycleTime::from_int(2))
}

#[test]
fn can_play_seq_of_three_units_at_seven_times_speed() {
    play_seq_of_three_units_with_multiplier(CycleTime::from_int(7))
}

#[test]
fn can_play_seq_of_unit_then_silence_then_unit() {
    // [  Seq  ]
    // ↓ ↓ ↓
    // A  ~  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let multiple_length = CycleTime::from_int(3);

    let make_scheduled_action = |start_time, letter: Letter| {
        [ScheduledExpectation {
            start_time: CycleTime::from_int(start_time) / multiple_length,
            duration: multiple_length.recip(),
            note_unit: note_unit(letter),
        }]
    };
    let interpreter_time =
        |start_time| CycleTime::from_int(start_time) / multiple_length;
    let expected_schedule_actions = [
        (interpreter_time(1), &make_scheduled_action(0, Letter::A)[..]),
        (interpreter_time(2), &[][..]),
        (interpreter_time(3), &make_scheduled_action(2, Letter::C)[..]),
        (interpreter_time(4), &make_scheduled_action(3, Letter::A)[..]),
        (interpreter_time(5), &[][..]),
        (interpreter_time(6), &make_scheduled_action(5, Letter::C)[..]),
    ];
    println!("Expected: {expected_schedule_actions:?}");
    let (arenas, head_index) =
        multiple_of_unit_then_silence_then_unit(Pattern::Seq);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
