use crate::ast::CycleInterval;
use crate::ast::CycleTime;
use crate::mem::Vec;
use crate::test::interpreter::ScheduledExpectation;

#[allow(unused)]
#[derive(Debug)]
pub struct NoteSequence {
    pub interval: CycleInterval,
    pub offset: CycleTime,
    pub multiplier: CycleTime,
    pub expected: Vec<ScheduledExpectation>,
}
