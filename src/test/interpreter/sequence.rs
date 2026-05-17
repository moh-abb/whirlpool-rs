use crate::alloc_types::Vec;
use crate::ast::time::CycleTime;
use crate::ast::time::CycleTimeInterval;
use crate::test::interpreter::ScheduledExpectation;

#[allow(unused)]
#[derive(Debug)]
pub struct NoteSequence {
    pub interval: CycleTimeInterval,
    pub offset: CycleTime,
    pub multiplier: CycleTime,
    pub expected: Vec<ScheduledExpectation>,
}
