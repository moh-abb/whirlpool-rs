use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::arbitrary::strategy::ArbitrarySequenceStrategy;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::interpreter::test_expectations_with_interpreter_setup_and_start_time;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::pattern::arenas::GrowableArenas;

pub struct PlayedUnitsMatchExpectations;
impl<Arenas: PatternArenas> ArenaTest<(Index<Pattern>, NoteSequence), Arenas>
    for PlayedUnitsMatchExpectations
{
    fn run(arenas: &Arenas, (index, sequence): (Index<Pattern>, NoteSequence)) {
        test_expectations_with_interpreter_setup_and_start_time(
            arenas,
            index,
            sequence.interval.start(),
            &[(sequence.interval.end(), sequence.expected)],
            FullInterpreterSetup {
                offset: sequence.offset,
                multiplier: sequence.multiplier,
            },
        );
    }
}

#[test]
fn played_units_match_expectations_once() {
    with_regenerated_arenas::<
        _,
        PlayedUnitsMatchExpectations,
        GrowableArenas,
        ArbitrarySequenceStrategy,
    >()
}

#[test]
fn played_units_match_expectations_multiple() {
    with_reused_arenas::<
        _,
        PlayedUnitsMatchExpectations,
        GrowableArenas,
        ArbitrarySequenceStrategy,
    >()
}
