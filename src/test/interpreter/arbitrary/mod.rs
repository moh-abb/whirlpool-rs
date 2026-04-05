use crate::ast::pattern::arenas::PatternArenas;
use crate::test::interpreter::FullInterpreterSetup;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::interpreter::test_expectations_with_interpreter_setup;
use crate::test::pattern::arena_alloc::ArenaTest;

mod expectations;
mod strategy;

pub struct PlayNoteSequence;
impl ArenaTest<NoteSequence> for PlayNoteSequence {
    fn run(arenas: &impl PatternArenas, sequence: NoteSequence) {
        test_expectations_with_interpreter_setup(
            arenas,
            sequence.head,
            &sequence.expected,
            FullInterpreterSetup {
                offset: sequence.offset,
                multiplier: sequence.multiplier,
            },
        );
    }
}
