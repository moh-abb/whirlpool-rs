use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::structures::index::Index;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::arbitrary::strategy::ArbitrarySequenceStrategy;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::interpreter::test_expectations_with_interpreter_setup_and_start_time;
use crate::test::pattern::arena_alloc::ArenaTest;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;
use crate::test::pattern::arena_alloc::with_reused_arenas;

mod expectations;
mod strategy;

pub struct PlayedUnitsMatchExpectations;
impl ArenaTest<(Index<Pattern>, NoteSequence)>
    for PlayedUnitsMatchExpectations
{
    fn run(
        arenas: &impl PatternArenas,
        (index, sequence): (Index<Pattern>, NoteSequence),
    ) {
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
