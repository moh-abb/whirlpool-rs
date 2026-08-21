use chumsky::Parser;
use proptest::strategy::Strategy;

use crate::ast::CycleTime;
use crate::ast::PatternNode;
use crate::ast::parser::Parseable;
use crate::ast::time::arbitrary::arb_cycle_time;
use crate::mem::GrowableArena;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::StrategyWithArena;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::mem::arenas_to::ArenasTo;

struct AnyTimeStrategy;
impl<Arenas> StrategyWithArena<CycleTime, Arenas> for AnyTimeStrategy {
    fn item_strategy() -> impl Strategy<Value = ArenasTo<Arenas, CycleTime>> {
        arb_cycle_time().prop_map(|time| ArenasTo::new(move |_arenas| Ok(time)))
    }
}

struct ValidateTime;
impl<Arenas> ArenaTest<CycleTime, Arenas> for ValidateTime {
    fn run(_arenas: &mut Arenas, time: CycleTime) {
        let formatted = format!("{time}");
        let parser = CycleTime::parser(());
        let result = parser
            .parse(formatted.as_bytes())
            .into_result()
            .expect("Parsing single CycleTime should succeed")
            .expect("Parsed CycleTime should not overflow");
        assert_eq!(
            result, time,
            "Parsed CycleTime should be equal to original"
        );
    }
}

#[test]
fn can_parse_cycle_time_once() {
    with_regenerated_arenas::<
        CycleTime,
        ValidateTime,
        GrowableArena<PatternNode>,
        AnyTimeStrategy,
    >();
}

#[test]
fn can_parse_cycle_time_multiple() {
    with_reused_arenas::<
        CycleTime,
        ValidateTime,
        GrowableArena<PatternNode>,
        AnyTimeStrategy,
    >();
}
