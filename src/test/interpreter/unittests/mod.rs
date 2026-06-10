use crate::ast::CycleTime;
use crate::ast::time::OverflowError;

mod cat;
mod seq;
mod silence;
mod stack;
mod timedstep;
mod unit;

// A complex (approximation of pi) offset cycle time to test played units.
const NONZERO_OFFSET: CycleTime = {
    let opt_offset = CycleTime::unwrapped_from_int(355)
        .div(CycleTime::unwrapped_from_int(113));
    match opt_offset {
        Err(OverflowError) => panic!("Overflow error"),
        Ok(offset) => offset,
    }
};
