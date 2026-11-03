use crate::interpreter::error::PatternInterpreterResult;
use crate::synth::unit::SoundUnit;

#[cfg_attr(test, mockall::automock)]
pub trait ValueStack {
    fn push_sound_unit(
        &mut self,
        sound_unit: SoundUnit,
    ) -> PatternInterpreterResult<()>;
}
