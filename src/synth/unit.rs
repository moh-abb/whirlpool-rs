use crate::ast::note::NoteUnit;
use crate::ast::time::CycleTime;

#[derive(derive_new::new, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SoundUnit {
    unit: NoteUnit,
    start: CycleTime,
    duration: CycleTime,
}

impl SoundUnit {
    pub fn unit(&self) -> NoteUnit {
        self.unit
    }

    pub fn start(&self) -> CycleTime {
        self.start
    }

    pub fn duration(&self) -> CycleTime {
        self.duration
    }
}
