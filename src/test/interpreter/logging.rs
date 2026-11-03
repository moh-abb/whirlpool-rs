use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::value::ValueStack;
use crate::synth::unit::SoundUnit;

pub struct LoggingValueStack<'a, ValStack> {
    scheduler: &'a mut ValStack,
    inner_enabled: bool,
    log_enabled: bool,
}

impl<'a, ValStack: ValueStack> LoggingValueStack<'a, ValStack> {
    pub fn new(scheduler: &'a mut ValStack) -> Self {
        Self { scheduler, inner_enabled: true, log_enabled: false }
    }

    pub fn set_inner_enabled(&mut self, inner_enabled: bool) {
        self.inner_enabled = inner_enabled;
    }

    pub fn set_logging(&mut self, log_enabled: bool) {
        self.log_enabled = log_enabled;
    }

    pub fn get_mut_scheduler(&mut self) -> &mut ValStack {
        &mut self.scheduler
    }
}

impl<'a, ValStack: ValueStack> ValueStack for LoggingValueStack<'a, ValStack> {
    fn push_sound_unit(
        &mut self,
        sound_unit: SoundUnit,
    ) -> PatternInterpreterResult<()> {
        if !self.inner_enabled {
            return Ok(());
        }

        if self.log_enabled {
            println!("LoggingScheduler: Playing {sound_unit:?}");
        }

        self.scheduler
            .push_sound_unit(sound_unit)
    }
}
