use crate::ast::note::Note;
use crate::ast::note::NoteLetter;
use crate::ast::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::synth::unit::SoundUnit;
use crate::test::ast::examples::pattern::one_cycle_unit;
use crate::test::interpreter::test_expectations;

#[test]
fn can_play_unit_for_one_cycle() {
    let note = Note::new(NoteLetter::A);
    // Test:
    // - Updating to one cycle will lead to exactly one invocation of
    // scheduling the note unit.
    // - Updating the cycle to the same time does nothing.
    let expectation =
        SoundUnit::new(NoteUnit::Note(note), CycleTime::ZERO, CycleTime::ONE);
    let expected_schedule_actions =
        [(CycleTime::ONE, &[expectation][..]), (CycleTime::ONE, &[][..])];
    let (arenas, head_index) = one_cycle_unit(note);
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
