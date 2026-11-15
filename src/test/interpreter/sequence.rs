use crate::ast::time::CycleTime;
use crate::ast::time::interval::CycleInterval;
use crate::mem::Vec;
use crate::synth::unit::SoundUnit;

#[allow(unused)]
#[derive(Debug)]
pub struct NoteSequence {
    pub interval: CycleInterval,
    pub offset: CycleTime,
    pub multiplier: CycleTime,
    pub expected: Vec<SoundUnit>,
}
