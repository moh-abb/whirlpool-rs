#![cfg(test)]

use proptest::arbitrary::any;
use proptest::prop_oneof;
use proptest::strategy::Strategy;

use crate::ast::note::Note;
use crate::ast::note::NoteNumber;
use crate::ast::note::NoteUnit;
use crate::ast::time::arbitrary::arb_cycle_time;

pub fn arb_note_number() -> impl Strategy<Value = NoteNumber> {
    arb_cycle_time().prop_map(NoteNumber)
}

pub fn arb_note_unit() -> impl Strategy<Value = NoteUnit> {
    prop_oneof![
        arb_note_number().prop_map(NoteUnit::Number),
        any::<Note>().prop_map(NoteUnit::WithOctave)
    ]
}
