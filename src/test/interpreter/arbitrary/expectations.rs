use core::num::NonZeroU8;
use core::num::NonZeroU16;

use proptest::prelude::Strategy;
use proptest::test_runner::Reason;

use crate::ast::time::CycleTime;
use crate::ast::time::arbitrary::arb_cycle_time;
use crate::ast::time::arbitrary::arb_positive_cycle_time;
use crate::ast::time::interval::CycleInterval;

const MAX_START_TIME: CycleTime = CycleTime::unwrapped_from_int(1 << 9);
const MAX_DURATION: CycleTime = CycleTime::unwrapped_from_int(1 << 5);
const MAX_MULTIPLIER: CycleTime = CycleTime::unwrapped_from_int(1 << 4);

/// Returns an arbitrary end time, offset and multiplier.
/// The maximum start time, duration and multiplier are chosen with the maximum
/// patten size to avoid the overflow case.
/// With a multiplier of 2^4, pattern size of ~64 = 2^6,
/// and maximum start time of 2^9
/// (with the duration substantially less than the start time),
/// we would expect to require 4 + 6 + 9 = 19 bits of precision.
/// This should fit within the 20 integer bits provided by [CycleTime].
#[allow(unused)]
pub fn arb_interval_offset_and_multiplier()
-> impl Strategy<Value = (CycleInterval, CycleTime, CycleTime)> {
    let arb_start_time = arb_positive_cycle_time::<NonZeroU16>().prop_filter(
        Reason::from("Start time should be at most {MAX_START_TIME:?}"),
        |time| time <= &MAX_START_TIME,
    );
    let arb_duration = arb_positive_cycle_time::<NonZeroU8>().prop_filter(
        Reason::from("Duration should be at most {MAX_DURATION:?}"),
        |time| time <= &MAX_DURATION,
    );
    let arb_interval = (arb_start_time, arb_duration).prop_filter_map(
        Reason::from("Overflow error"),
        |(start, duration)| {
            Some(CycleInterval::new(start, start.add(duration).ok()?))
        },
    );
    let arb_multiplier = arb_positive_cycle_time::<NonZeroU8>().prop_filter(
        Reason::from("Duration should be at most {MAX_MULTIPLIER:?}"),
        |time| time <= &MAX_MULTIPLIER,
    );
    (arb_interval, arb_cycle_time(), arb_multiplier)
}
