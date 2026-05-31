use crate::ast::CycleTime;
use crate::ast::NoteFrequency;
use crate::ast::NoteUnit;
use crate::test::interpreter::ScheduledExpectation;
use crate::test::interpreter::test_expectations;
use crate::test::interpreter::unittests::examples::one_cycle_unit;

#[test]
fn can_play_unit_for_one_cycle() {
    let note_unit = NoteUnit::Frequency(NoteFrequency(440));
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
