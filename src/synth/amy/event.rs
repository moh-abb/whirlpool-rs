use crate::synth::amy::amy_ext;

#[allow(unused)]
pub struct AmyEvent(amy_ext::amy_event);

impl AmyEvent {
    #[allow(unused)]
    pub fn new() -> Self {
        Self(unsafe { amy_ext::amy_default_event() })
    }
}
