use crate::ast::CycleTime;
use crate::test::examples::unittests::pattern::one_cycle_silence;
use crate::test::interpreter::test_expectations;

#[test]
fn can_play_silence_for_one_cycle() {
    let expected_schedule_actions = [
        (CycleTime::unwrapped_from_int(1), &[][..]),
        (CycleTime::unwrapped_from_int(2), &[][..]),
    ];
    let (arenas, head_index) = one_cycle_silence();
    test_expectations(&arenas, head_index, &expected_schedule_actions);
}
