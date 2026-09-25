use combine::EasyParser;
use proptest::strategy::Strategy;

use crate::ast::pattern::PatternNode;
use crate::ast::time::CycleTime;
use crate::ast::time::arbitrary::arb_cycle_time;
use crate::mem::ArenaError;
use crate::mem::GrowableArena;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::ArenasTo;
use crate::mem::arena::test::StrategyWithArena;
use crate::mem::arena::test::arenas_test_run;

struct AnyTimeStrategy;
impl<Arenas> StrategyWithArena<CycleTime, Arenas, ArenaError>
    for AnyTimeStrategy
{
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, CycleTime, ArenaError>>
    where
        Arenas: 'r,
    {
        arb_cycle_time().prop_map(|time| ArenasTo::new(move |_arenas| Ok(time)))
    }
}

struct ValidateTime;
impl<Arenas> ArenaTest<CycleTime, Arenas> for ValidateTime {
    fn run<'r>(_arenas: SharedArenaRef<'r, Arenas>, time: CycleTime) {
        let formatted = format!("{time}");
        let mut parser = CycleTime::parser();
        let result = parser
            .easy_parse(formatted.as_bytes())
            .expect("Parsing single CycleTime should succeed")
            .0
            .expect("Parsed CycleTime should not overflow");
        assert_eq!(
            result, time,
            "Parsed CycleTime should be equal to original"
        );
    }
}

#[test]
fn can_parse_cycle_time() {
    arenas_test_run::<
        CycleTime,
        ValidateTime,
        GrowableArena<PatternNode>,
        AnyTimeStrategy,
        ArenaError,
    >();
}
