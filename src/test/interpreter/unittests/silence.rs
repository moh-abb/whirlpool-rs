use crate::ast::CycleTime;
use crate::test::interpreter::test_expectations;
use crate::test::interpreter::unittests::examples::one_cycle_silence;

#[test]
fn can_play_silence_for_one_cycle() {
    let expected_schedule_actions =
        [(CycleTime::from_int(1), &[][..]), (CycleTime::from_int(2), &[][..])];
    let (arenas, head_index) = one_cycle_silence();
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
