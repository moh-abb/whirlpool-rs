use crate::ast::time::CycleTime;
use crate::synth::unit::SoundUnit;

#[cfg_attr(test, mockall::automock)]
pub trait UnitScheduler {
    fn add(&mut self, sound: SoundUnit, start: CycleTime);
}
