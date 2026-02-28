use crate::alloc_types::Vec;
use crate::ast::pattern::Pattern;
use crate::ast::time::CycleTime;
use crate::structures::index::Index;
use crate::test::interpreter::ScheduledExpectation;

#[allow(unused)]
#[derive(Debug)]
pub struct NoteSequence {
    pub head: Index<Pattern>,
    pub offset: CycleTime,
    pub multiplier: CycleTime,
    pub expected: Vec<(CycleTime, Vec<ScheduledExpectation>)>,
}
