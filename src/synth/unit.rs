use crate::ast::CycleTime;
use crate::ast::NoteUnit;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SoundUnit {
    unit: NoteUnit,
    duration: CycleTime,
}

impl SoundUnit {
    pub fn new(unit: NoteUnit, duration: CycleTime) -> Self {
        Self { unit, duration }
    }

    pub fn unit(&self) -> NoteUnit {
        self.unit
    }

    pub fn duration(&self) -> CycleTime {
        self.duration
    }
}
