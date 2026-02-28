use core::num::NonZeroU16;

use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prop_oneof;
use proptest::test_runner::Reason;

use crate::ast::time::CycleTime;

#[allow(unused)]
pub fn arb_positive_cycle_time() -> impl Strategy<Value = CycleTime> {
    (any::<NonZeroU16>(), any::<NonZeroU16>()).prop_filter_map(
        Reason::from("result should be positive"),
        |(num, denom)| {
            let num_time = CycleTime::from_int(num.get().into());
            let denom_time = CycleTime::from_int_recip(denom.get().into());
            let result = Some(num_time.mul(denom_time));
            result.filter(|x| x > &CycleTime::ZERO)
        },
    )
}

#[allow(unused)]
pub fn arb_cycle_time() -> impl Strategy<Value = CycleTime> {
    prop_oneof![
        Just(CycleTime::ZERO),
        arb_positive_cycle_time(),
        arb_positive_cycle_time().prop_map(CycleTime::neg),
    ]
}
