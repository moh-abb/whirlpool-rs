use mockall::predicate;

use crate::ast::time::CycleTime;
use crate::interpreter::value::MockValueStack;
use crate::interpreter::value::Value;
use crate::interpreter::value::ValueStack;
use crate::interpreter::value::ValueStackError;
use crate::synth::unit::SoundUnit;

pub struct LoggingValueStack {
    mock: MockValueStack,
    real: Vec<Value>,
    inner_enabled: bool,
    log_enabled: bool,
}

impl LoggingValueStack {
    pub fn new() -> Self {
        Self {
            mock: MockValueStack::new(),
            real: Vec::new(),
            inner_enabled: true,
            log_enabled: false,
        }
    }

    pub fn set_inner_enabled(&mut self, inner_enabled: bool) {
        if self.log_enabled {
            println!("Setting inner enabled to {inner_enabled}");
        }

        self.inner_enabled = inner_enabled;
    }

    pub fn set_logging_enabled(&mut self, log_enabled: bool) {
        if self.log_enabled {
            println!("Setting log enabled to {log_enabled}");
        }

        self.log_enabled = log_enabled;
    }

    pub fn expect_sound_unit(&mut self, sound_unit: SoundUnit) {
        assert!(self.inner_enabled);

        if self.log_enabled {
            println!("Adding expectation {sound_unit:?}");
        }

        let abs_diff = |x: CycleTime, y: CycleTime| {
            if x <= y { y - x } else { x - y }
        };
        let approx_eq_unit = predicate::function(move |unit: &SoundUnit| {
            unit.unit() == sound_unit.unit()
                && abs_diff(unit.duration(), sound_unit.duration())
                    <= CycleTime::EPSILON
                && abs_diff(unit.start(), sound_unit.start())
                    <= CycleTime::EPSILON
        });
        self.mock
            .expect_push()
            .with(approx_eq_unit)
            .once()
            .returning(|_| Ok(()));
    }

    pub fn checkpoint(&mut self) {
        if self.log_enabled {
            println!("Checkpointing");
        }

        self.mock.checkpoint();
    }
}

impl ValueStack for LoggingValueStack {
    fn push<T>(&mut self, item: T) -> Result<(), ValueStackError>
    where
        T: Into<Value> + 'static,
    {
        let value = item.into();
        if self.log_enabled {
            println!("Pushing {value:?}");
        }

        match value {
            Value::SoundUnit(sound_unit) if self.inner_enabled => {
                if self.log_enabled {
                    println!("Pushing {sound_unit:?} via mock")
                }
                self.mock.push(sound_unit)
            }
            other => Ok(self.real.push(other)),
        }
    }

    fn pop<T>(&mut self) -> Result<T, ValueStackError>
    where
        T: 'static,
        Value: TryInto<T>,
    {
        if self.log_enabled {
            println!("Popping value");
        }

        // We only expect to non-sound-unit values for now.
        let popped_value = self
            .real
            .pop()
            .ok_or(ValueStackError::StackEmpty)?;

        if self.log_enabled {
            println!("Popped {popped_value:?}");
        }

        popped_value
            .try_into()
            .map_err(|_| ValueStackError::TypeMismatch)
    }
}
