use core::cell::RefCell;

use mockall::predicate;

use crate::arena::Arena;
use crate::arena::arena_impl::fixed_arena::FixedArena;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::Interpreter;
use crate::ast::pattern::note::Frequency;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::player::pattern::MockPatternPlayer;
use crate::player::pattern::SoundUnit;

#[derive(Debug, Default)]
struct FixedArenas {
    pattern_arena: FixedArena<Pattern>,
    pattern_chain_arena: FixedArena<Chain<Pattern>>,
    timed_step_arena: FixedArena<TimedStep>,
    timed_step_chain_arena: FixedArena<Chain<TimedStep>>,
}

impl PatternArenas for FixedArenas {
    fn get_pattern_arena(&self) -> &impl Arena<Pattern> {
        &self.pattern_arena
    }

    fn get_pattern_chain_arena(&self) -> &impl Arena<Chain<Pattern>> {
        &self.pattern_chain_arena
    }

    fn get_timed_step_arena(&self) -> &impl Arena<TimedStep> {
        &self.timed_step_arena
    }

    fn get_timed_step_chain_arena(&self) -> &impl Arena<Chain<TimedStep>> {
        &self.timed_step_chain_arena
    }
}

/// A triple of an expected scheduled start time, duration, and note unit.
type ScheduledExpectation = (CycleTime, CycleTime, NoteUnit);

/// Performs a simulation on a pattern stored in `arenas` with given head index
/// `head_index`. For each element in `expected_actions`, the first denotes
/// the next time to update, and the second denotes a list of expected scheduled
/// events: the scheduled time, duration, and unit. The expectation will be
/// added and checked sequentially for each pair in `expected_actions`.
fn test_fixed_arenas(
    arenas: FixedArenas,
    head_index: Index<Pattern>,
    expected_schedule_actions: &[(CycleTime, &[ScheduledExpectation])],
) {
    let mock_player = RefCell::new(MockPatternPlayer::new());
    let mut interpreter =
        Interpreter::new_with_refcell(head_index, &arenas, &mock_player);

    let expect_note_unit = |expectation: ScheduledExpectation| {
        let (start_time, duration, note_unit) = expectation;
        let sound_unit = SoundUnit::new(note_unit, duration);
        mock_player
            .borrow_mut()
            .expect_schedule_note_unit()
            .with(predicate::eq(sound_unit), predicate::eq(start_time))
            .return_const(());
    };

    for &(next_cycle_time, expectations) in expected_schedule_actions {
        expectations
            .iter()
            .cloned()
            .for_each(expect_note_unit);
        interpreter.update_time(next_cycle_time);
        mock_player.borrow_mut().checkpoint();
    }
}

#[test]
fn can_play_unit_for_one_cycle() {
    let note_unit = NoteUnit::Frequency(Frequency(440));
    let head_index = Index::new(0);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([(
            head_index.clone(),
            Pattern::Note(note_unit),
        )]),
        ..Default::default()
    };
    // Test:
    // - Updating to one cycle will lead to exactly one invocation of
    // scheduling the note unit.
    // - Updating the cycle to the same time does nothing.
    let expected_schedule_actions = [
        (CycleTime::ONE, &[(CycleTime::ZERO, CycleTime::ONE, note_unit)][..]),
        (CycleTime::ONE, &[][..]),
    ];
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_double_alternating_cats() {
    // [ Cat ] -----v
    //    ↓       ↓
    // [ Cat ]  [ Cat ]
    // ↓  ↓   ↓  ↓
    // A   B    C   D
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let first_cat = Pattern::Cat(Multiple::new_nonempty(
        2,
        Index::new(0xC0),
        Index::new(0xC1),
    ));
    let second_cat = Pattern::Cat(Multiple::new_nonempty(
        2,
        Index::new(0xC2),
        Index::new(0xC3),
    ));
    let overall_cat = Pattern::Cat(Multiple::new_nonempty(
        2,
        Index::new(0xC4),
        Index::new(0xC5),
    ));
    let head_index = Index::new(6);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(2), Pattern::Note(note_unit(Letter::C))),
            (Index::new(3), Pattern::Note(note_unit(Letter::D))),
            (Index::new(4), first_cat),
            (Index::new(5), second_cat),
            (head_index.clone(), overall_cat),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(Index::new(1), Some(Index::new(0xC0)), None),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(2), None, Some(Index::new(0xC3))),
            ),
            (
                Index::new(0xC3),
                Chain(Index::new(3), Some(Index::new(0xC2)), None),
            ),
            (
                Index::new(0xC4),
                Chain(Index::new(4), None, Some(Index::new(0xC5))),
            ),
            (
                Index::new(0xC5),
                Chain(Index::new(5), Some(Index::new(0xC4)), None),
            ),
        ]),
        ..Default::default()
    };

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
    let first_cat = Pattern::Cat(Multiple::new_nonempty(
        2,
        Index::new(0xC0),
        Index::new(0xC1),
    ));
    let overall_cat = Pattern::Cat(Multiple::new_nonempty(
        2,
        Index::new(0xC2),
        Index::new(0xC3),
    ));
    let head_index = Index::new(6);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(4), first_cat),
            (Index::new(5), Pattern::Note(note_unit(Letter::C))),
            (head_index.clone(), overall_cat),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(Index::new(1), Some(Index::new(0xC0)), None),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(4), None, Some(Index::new(0xC3))),
            ),
            (
                Index::new(0xC3),
                Chain(Index::new(5), Some(Index::new(0xC2)), None),
            ),
        ]),
        ..Default::default()
    };

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
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}

#[test]
fn can_play_cat_of_three_units() {
    // [  Cat  ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let overall_cat = Pattern::Cat(Multiple::new_nonempty(
        3,
        Index::new(0xC0),
        Index::new(0xC2),
    ));
    let head_index = Index::new(3);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(2), Pattern::Note(note_unit(Letter::C))),
            (head_index.clone(), overall_cat),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(
                    Index::new(1),
                    Some(Index::new(0xC0)),
                    Some(Index::new(0xC2)),
                ),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(2), Some(Index::new(0xC1)), None),
            ),
        ]),
        ..Default::default()
    };

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
    test_fixed_arenas(arenas, head_index, &expected_schedule_actions);
}
