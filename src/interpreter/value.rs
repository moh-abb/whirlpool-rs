use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::props::ElemProps;
use crate::synth::unit::SoundUnit;

#[cfg_attr(test, mockall::automock)]
pub trait ValueStack {
    fn push_sound_unit(
        &mut self,
        sound_unit: SoundUnit,
    ) -> PatternInterpreterResult<()>;

    fn push_total_props(
        &mut self,
        total_props: ElemProps<()>,
    ) -> PatternInterpreterResult<()>;

    fn pop_total_props(&mut self) -> PatternInterpreterResult<ElemProps<()>>;
}
