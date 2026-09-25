use combine::EasyParser;
use proptest::strategy::Strategy;

use crate::ast::note::NoteUnit;
use crate::ast::note::arbitrary::arb_note_unit;
use crate::ast::pattern::PatternNode;
use crate::mem::ArenaError;
use crate::mem::GrowableArena;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::ArenasTo;
use crate::mem::arena::test::StrategyWithArena;
use crate::mem::arena::test::arenas_test_run;

struct AnyNoteUnitStrategy;
impl<Arenas> StrategyWithArena<NoteUnit, Arenas, ArenaError>
    for AnyNoteUnitStrategy
{
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, NoteUnit, ArenaError>>
    where
        Arenas: 'r,
    {
        arb_note_unit()
            .prop_map(|note_unit| ArenasTo::new(move |_arenas| Ok(note_unit)))
    }
}

struct ValidateNoteUnit;
impl<Arenas> ArenaTest<NoteUnit, Arenas> for ValidateNoteUnit {
    fn run<'r>(_arenas: SharedArenaRef<'r, Arenas>, note_unit: NoteUnit) {
        let formatted = format!("{note_unit}");
        let mut parser = NoteUnit::parser();
        let result = parser
            .easy_parse(formatted.as_bytes())
            .expect("Parsing single note unit should succeed")
            .0
            .expect("Parsed note unit should not error");
        assert_eq!(
            result, note_unit,
            "Parsed note unit {result} should be equal to original {note_unit}"
        );
    }
}

#[test]
fn can_parse_note_unit() {
    arenas_test_run::<
        NoteUnit,
        ValidateNoteUnit,
        GrowableArena<PatternNode>,
        AnyNoteUnitStrategy,
        ArenaError,
    >();
}
