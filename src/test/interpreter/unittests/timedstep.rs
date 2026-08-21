//! Unit tests for [Pattern::Arrange] and [Pattern::TimeCat].

use crate::ast::CycleTime;
use crate::ast::Note;
use crate::ast::NoteLetter;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::time::OverflowError;
use crate::test::examples::unittests::pattern::half_binary_tree_depth_two_with_timed_steps;
use crate::test::examples::unittests::pattern::multiple_of_four_timed_steps;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::test_expectations;

fn play_nested_time_cats_with_elem_lengths(
    root_elem_lengths: [CycleTime; 2],
    child_elem_lengths: [CycleTime; 2],
) {
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));
    let proportion_of_root = |index: usize| {
        root_elem_lengths[index]
            / root_elem_lengths
                .into_iter()
                .sum::<Result<CycleTime, OverflowError>>()
                .unwrap()
    };
    let proportion_of_children = |index: usize| {
        child_elem_lengths[index]
            / child_elem_lengths
                .into_iter()
                .sum::<Result<CycleTime, OverflowError>>()
                .unwrap()
    };
    let a_plus_b_length = proportion_of_root(0);
    let unit_len = |letter: NoteLetter| match letter {
        NoteLetter::A => a_plus_b_length * proportion_of_children(0),
        NoteLetter::B => a_plus_b_length * proportion_of_children(1),
        NoteLetter::C => proportion_of_root(1),
        _ => unreachable!(),
    };
    let expectation = |start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time,
            duration: unit_len(letter),
            note_unit: note_unit(letter),
        }]
    };
    let mut expected_schedule_actions = Vec::new();
    // Check expectations for up to two whole repetitions
    let max_time = 2;
    for rep in 0..max_time {
        let rep_time = CycleTime::unwrapped_from_int(rep);
        expected_schedule_actions.extend([
            (
                rep_time + unit_len(NoteLetter::A),
                expectation(rep_time, NoteLetter::A),
            ),
            (
                rep_time + a_plus_b_length,
                expectation(rep_time + unit_len(NoteLetter::A), NoteLetter::B),
            ),
            (
                rep_time + CycleTime::ONE,
                expectation(rep_time + a_plus_b_length, NoteLetter::C),
            ),
        ]);
    }

    let make_time_cat = |total_cycle_length, multiple| Pattern::TimeCat {
        total_cycle_length,
        multiple,
    };
    let (arenas, head_index) = half_binary_tree_depth_two_with_timed_steps(
        make_time_cat,
        make_time_cat,
        root_elem_lengths,
        child_elem_lengths,
    );
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_nested_time_cats_with_equal_elem_lengths() {
    play_nested_time_cats_with_elem_lengths(
        [CycleTime::ONE; 2],
        [CycleTime::ONE; 2],
    );
}

#[test]
fn can_play_nested_time_cats_with_nonequal_root_lengths() {
    play_nested_time_cats_with_elem_lengths(
        [CycleTime::ONE, CycleTime::unwrapped_from_int(3)],
        [CycleTime::ONE; 2],
    );
}

#[test]
fn can_play_nested_time_cats_with_nonequal_child_lengths() {
    play_nested_time_cats_with_elem_lengths(
        [CycleTime::ONE; 2],
        [CycleTime::ONE, CycleTime::unwrapped_from_int(3)],
    );
}

fn play_linear_time_cat_with_elem_lengths(elem_lengths: [CycleTime; 4]) {
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));
    let accum_lengths = elem_lengths.iter().copied().fold(
        vec![CycleTime::ZERO],
        |mut arr, x| {
            arr.push(*arr.last().unwrap() + x);
            arr
        },
    );
    let total = *accum_lengths.last().unwrap();
    let elem_duration = |index: usize| elem_lengths[index] / total;
    let unit_len = |letter: NoteLetter| match letter {
        NoteLetter::A => elem_duration(0),
        NoteLetter::B => elem_duration(1),
        NoteLetter::C => elem_duration(2),
        NoteLetter::D => elem_duration(3),
        _ => unreachable!(),
    };
    let expectation = |start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time,
            duration: unit_len(letter),
            note_unit: note_unit(letter),
        }]
    };
    // Check expectations for up to two whole repetitions
    let max_time = 2;
    let mut expected_schedule_actions =
        Vec::<(CycleTime, Vec<ScheduledExpectation>)>::new();
    for rep in 0..max_time {
        let rep_time = CycleTime::unwrapped_from_int(rep);
        let letters =
            [NoteLetter::A, NoteLetter::B, NoteLetter::C, NoteLetter::D];
        let mut elems = Vec::new();
        let mut cur_time = rep_time;
        for letter in letters {
            let next_time = cur_time + unit_len(letter);
            elems.push((
                cur_time + unit_len(letter),
                expectation(cur_time, letter).to_vec(),
            ));
            cur_time = next_time;
        }
        expected_schedule_actions.extend(elems);
    }

    let make_time_cat = |total_cycle_length, multiple| Pattern::TimeCat {
        total_cycle_length,
        multiple,
    };
    let (arenas, head_index) =
        multiple_of_four_timed_steps(make_time_cat, elem_lengths);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_linear_time_cat_with_elem_lengths_of_one() {
    play_linear_time_cat_with_elem_lengths([CycleTime::ONE; 4])
}

#[test]
fn can_play_linear_time_cat_with_elem_lengths_of_half() {
    play_linear_time_cat_with_elem_lengths([CycleTime::from_int_recip(2); 4])
}

#[test]
fn can_play_linear_time_cat_with_elem_lengths_of_two() {
    play_linear_time_cat_with_elem_lengths(
        [CycleTime::unwrapped_from_int(2); 4],
    )
}

#[test]
fn can_play_linear_time_cat_with_elem_lengths_of_seven() {
    play_linear_time_cat_with_elem_lengths(
        [CycleTime::unwrapped_from_int(7); 4],
    )
}

#[test]
fn can_play_linear_time_cat_with_unequal_lengths_adding_to_one() {
    play_linear_time_cat_with_elem_lengths(
        [4, 4, 2, 2].map(CycleTime::from_int_recip),
    )
}

#[test]
fn can_play_linear_time_cat_with_unequal_lengths_not_adding_to_one() {
    play_linear_time_cat_with_elem_lengths(
        [4, 4, 2, 2].map(CycleTime::unwrapped_from_int),
    )
}

#[test]
fn can_play_linear_time_cat_with_complex_unequal_integer_lengths() {
    play_linear_time_cat_with_elem_lengths(
        [11, 13, 17, 23].map(CycleTime::unwrapped_from_int),
    )
}

#[test]
fn can_play_linear_time_cat_with_complex_unequal_decimal_lengths() {
    play_linear_time_cat_with_elem_lengths(
        [11, 13, 17, 23].map(CycleTime::from_int_recip),
    )
}

#[test]
fn can_play_nested_arranges() {
    let note_unit =
        |letter: NoteLetter| NoteUnit::WithOctave(Note::new(letter));
    let start_time = |num: i32| CycleTime::unwrapped_from_int(num);
    let single_action = |unscaled_start_time, letter: NoteLetter| {
        [ScheduledExpectation {
            start_time: start_time(unscaled_start_time),
            duration: CycleTime::ONE,
            note_unit: note_unit(letter),
        }]
    };
    let expected_schedule_actions = [
        (start_time(01), &single_action(00, NoteLetter::A)[..]),
        (start_time(02), &single_action(01, NoteLetter::C)[..]),
        (start_time(03), &single_action(02, NoteLetter::C)[..]),
        (start_time(04), &single_action(03, NoteLetter::B)[..]),
        (start_time(05), &single_action(04, NoteLetter::C)[..]),
        (start_time(06), &single_action(05, NoteLetter::C)[..]),
        (start_time(07), &single_action(06, NoteLetter::A)[..]),
        (start_time(08), &single_action(07, NoteLetter::C)[..]),
        (start_time(09), &single_action(08, NoteLetter::C)[..]),
        (start_time(10), &single_action(09, NoteLetter::B)[..]),
        (start_time(11), &single_action(10, NoteLetter::C)[..]),
        (start_time(12), &single_action(11, NoteLetter::C)[..]),
    ];

    let make_arrange = |total_cycle_length, multiple| Pattern::Arrange {
        total_cycle_length,
        multiple,
    };
    let (arenas, head_index) = half_binary_tree_depth_two_with_timed_steps(
        make_arrange,
        make_arrange,
        [CycleTime::ONE, CycleTime::unwrapped_from_int(2)],
        [CycleTime::ONE, CycleTime::ONE],
    );
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
