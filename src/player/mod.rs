use crate::ast::time::CycleTime;
use crate::player::unit::SoundUnit;

pub mod unit;

#[cfg_attr(test, mockall::automock)]
pub trait PatternPlayer {
    fn schedule_note_unit(&mut self, sound: SoundUnit, start: CycleTime);
}