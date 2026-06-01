use crate::ast::CycleTime;

mod cat;
mod seq;
mod silence;
mod stack;
mod timedstep;
mod unit;

// A complex (approximation of pi) offset cycle time to test played units.
const NONZERO_OFFSET: CycleTime =
    CycleTime::from_int(355).div(CycleTime::from_int(113));
