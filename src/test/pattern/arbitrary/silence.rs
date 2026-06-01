use proptest::prelude::Just;
use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prop_oneof;

use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::Index;
use crate::test::mem::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::unit::arb_pattern_note_unit;

fn arb_silence<Arenas: PatternArenas>() -> ArenasTo<Arenas, Index<Pattern>> {
    ArenasTo::new(|arenas: &Arenas| {
        arenas
            .get_pattern_arena()
            .push(Pattern::Silence)
    })
}

pub fn arb_pattern_leaf<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<Pattern>>> {
    prop_oneof![
        Just(arb_silence()),
        any::<NoteUnit>().prop_map(arb_pattern_note_unit)
    ]
}
