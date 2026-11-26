#![cfg(test)]

use proptest::arbitrary::any;
use proptest::prop_oneof;
use proptest::strategy::Strategy;

use crate::ast::note::Note;
use crate::ast::note::NoteUnit;

pub fn arb_note_unit() -> impl Strategy<Value = NoteUnit> {
    prop_oneof![any::<Note>().prop_map(NoteUnit::Note)]
}
