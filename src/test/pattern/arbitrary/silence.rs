use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prop_oneof;

use crate::arena::Arena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::structures::index::Index;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::unit::arb_pattern_note_unit;

fn arb_silence<Arenas: PatternArenas>() -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(|arenas: &Arenas| {
        arenas
            .get_pattern_arena()
            .alloc(Pattern::Silence)
    })
}

pub fn arb_pattern_leaf<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    prop_oneof![
        Just(arb_silence()),
        any::<NoteUnit>().prop_map(arb_pattern_note_unit)
    ]
}
