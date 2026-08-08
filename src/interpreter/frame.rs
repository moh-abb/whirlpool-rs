use core::ops::ControlFlow;

pub enum InterpreterFrame {}

/// Indicates the result of interpreting one frame.
/// If we return `Err(err)` then this indicates that interpreting has failed
/// with the given error. Otherwise, `Ok(Continue(None))` indicates to
/// continue stepping without pushing any more frames onto the stack, and
/// `Ok(Continue(Some(frame)))` indicates that `frame` should be pushed onto
/// the stack.
pub type EvaluateResult<Error> =
    Result<ControlFlow<(), Option<InterpreterFrame>>, Error>;

pub trait EvaluateFrame {
    type Error;

    fn step(&mut self) -> EvaluateResult<Self::Error>;
}
