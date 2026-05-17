use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::Index;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;

pub fn arb_pattern_note_unit<Arenas: PatternArenas>(
    unit: NoteUnit,
) -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(move |arenas: &Arenas| {
        arenas
            .get_pattern_arena()
            .alloc(Pattern::Note(unit))
    })
}
