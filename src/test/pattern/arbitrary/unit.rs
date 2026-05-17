use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::mem::arena::Arena;
use crate::structures::index::Index;
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
