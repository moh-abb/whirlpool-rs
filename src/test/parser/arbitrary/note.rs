use proptest::arbitrary::any;
use proptest::strategy::Strategy;

use crate::ast::expr::ExprNode;
use crate::ast::note::Note;
use crate::ast::note::NoteUnit;
use crate::mem::ArenaError;
use crate::mem::GrowableArena;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::ArenasTo;
use crate::mem::arena::test::StrategyWithArena;
use crate::mem::arena::test::arenas_test_run;

struct AnyNoteStrategy;
impl<Arenas> StrategyWithArena<Note, Arenas, ArenaError> for AnyNoteStrategy {
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, Note, ArenaError>>
    where
        Arenas: 'r,
    {
        any::<Note>()
            .prop_map(|note| ArenasTo::new(move |_arenas| Ok(note.clone())))
    }
}

struct ValidateNote;
impl<Arenas> ArenaTest<Note, Arenas> for ValidateNote {
    fn run<'r>(_arenas: SharedArenaRef<'r, Arenas>, note: Note) {
        let formatted = format!("{}", NoteUnit::Note(note));
        let result = Note::resolve(formatted.as_bytes())
            .expect("Parsed note unit should not error");
        assert_eq!(
            result, note,
            "Parsed note unit {result:?} should be equal to original {note:?}"
        );
    }
}

#[test]
fn can_parse_note_unit() {
    arenas_test_run::<
        Note,
        ValidateNote,
        GrowableArena<ExprNode>,
        AnyNoteStrategy,
        ArenaError,
    >();
}
