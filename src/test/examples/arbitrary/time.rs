use core::num::NonZeroU16;

use proptest::prelude::Arbitrary;
use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prop_oneof;
use proptest::test_runner::Reason;

use crate::ast::CycleTime;

pub fn arb_positive_cycle_time<NumSource: Arbitrary + Into<NonZeroU16>>()
-> impl Strategy<Value = CycleTime> {
    (any::<NumSource>(), any::<NumSource>()).prop_filter_map(
        Reason::from("result should be positive"),
        |(num, denom)| {
            let num_time = CycleTime::from_int(i32::from(num.into().get()));
            let denom_time =
                CycleTime::from_int_recip(i32::from(denom.into().get()));
            let result = Some(num_time * denom_time);
            result.filter(|x| x > &CycleTime::ZERO)
        },
    )
}

pub fn arb_cycle_time() -> impl Strategy<Value = CycleTime> {
    prop_oneof![
        Just(CycleTime::ZERO),
        arb_positive_cycle_time::<NonZeroU16>(),
        arb_positive_cycle_time::<NonZeroU16>().prop_map(CycleTime::neg),
    ]
}
