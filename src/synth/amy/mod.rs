#![allow(unsafe_code)]
/// Defines the Amy adapter, whose synth will be used for the audio engine.
///
/// This is one of the main regions in our codebase which uses unsafe code.
use spin::Mutex;
use spin::MutexGuard;

#[allow(unused)]
#[allow(unnecessary_transmutes)]
#[allow(unused_qualifications)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::all)]
mod amy_ext {
    include!(concat!(env!("OUT_DIR"), "/amy_bindings.rs"));
}

mod event;

static AMY_ADAPTER_SEMA: Mutex<()> = Mutex::new(());

/// Used to enforce single-threaded usage of the Amy library.
pub type Token = MutexGuard<'static, ()>;

pub struct AmyAdapter {
    _thread_token: Token,
}

pub enum AmyError {
    AlreadyCreated,
}

pub type AmyResult<T> = Result<T, AmyError>;

impl AmyAdapter {
    pub fn new() -> AmyResult<Self> {
        let _thread_token = AMY_ADAPTER_SEMA
            .try_lock()
            .ok_or(AmyError::AlreadyCreated)?;
        Ok(Self { _thread_token })
    }
}
