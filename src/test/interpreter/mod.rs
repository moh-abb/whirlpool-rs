use core::cell::RefCell;

use mockall::predicate;

use crate::structures::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::Interpreter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::player::MockPatternPlayer;
use crate::player::unit::SoundUnit;

mod cat_silence;
mod examples;
mod stack;

/// A triple of an expected scheduled start time, duration, and note unit.
type ScheduledExpectation = (CycleTime, CycleTime, NoteUnit);

/// Performs a simulation on a pattern stored in `arenas` with given head index
/// `head_index`. For each element in `expected_actions`, the first denotes
/// the next time to update, and the second denotes a list of expected scheduled
/// events: the scheduled time, duration, and unit. The expectation will be
/// added and checked sequentially for each pair in `expected_actions`.
fn test_fixed_arenas(
    arenas: impl PatternArenas,
    head_index: Index<Pattern>,
    expected_schedule_actions: &[(CycleTime, &[ScheduledExpectation])],
) {
    let mock_player = RefCell::new(MockPatternPlayer::new());
    let mut interpreter =
        Interpreter::new_with_refcell(head_index, &arenas, &mock_player);

    let expect_note_unit =
        |expectation: ScheduledExpectation, all: &[ScheduledExpectation]| {
            let (start_time, duration, note_unit) = expectation;
            let sound_unit = SoundUnit::new(note_unit, duration);
            let count = all
                .iter()
                .filter(|&&e| e == expectation)
                .count();
            mock_player
                .borrow_mut()
                .expect_schedule_note_unit()
                .times(count)
                .with(predicate::eq(sound_unit), predicate::eq(start_time))
                .return_const(());
        };

    for &(next_cycle_time, expectations) in expected_schedule_actions {
        expectations
            .iter()
            .cloned()
            .for_each(|e| expect_note_unit(e, expectations));
        interpreter.update_time(next_cycle_time);
        mock_player.borrow_mut().checkpoint();
    }
}
