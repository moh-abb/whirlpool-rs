use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SoundUnit {
    unit: NoteUnit,
    duration: CycleTime,
}

impl SoundUnit {
    pub fn new(unit: NoteUnit, duration: CycleTime) -> Self {
        Self { unit, duration }
    }
}
