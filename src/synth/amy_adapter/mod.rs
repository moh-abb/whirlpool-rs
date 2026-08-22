use spin::Mutex;
use spin::MutexGuard;

use crate::synth::amy_bindings as amy;

static AMY_ADAPTER_SEMA: Mutex<()> = Mutex::new(());

pub struct AmyAdapter {
    mutex_guard: MutexGuard<'static, ()>,
}

pub enum AmyError {
    AlreadyCreated,
}

pub type AmyResult<T> = Result<T, AmyError>;

impl AmyAdapter {
    pub fn new() -> AmyResult<Self> {
        let mutex_guard = AMY_ADAPTER_SEMA
            .try_lock()
            .ok_or(AmyError::AlreadyCreated)?;
        Ok(Self { mutex_guard })
    }
}
