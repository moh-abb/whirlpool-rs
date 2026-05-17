use crate::ast::time::CycleTime;

mod cat;
mod examples;
mod seq;
mod stack;
mod timedstep;
mod unit_silence;

// A complex (approximation of pi) offset cycle time to test played units.
const NONZERO_OFFSET: CycleTime =
    CycleTime::from_int(355).div(CycleTime::from_int(113));
