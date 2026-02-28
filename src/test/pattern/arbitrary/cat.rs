use proptest::prelude::Strategy;

use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::structures::index::Index;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::multiple::arb_pattern_with_units;

#[allow(unused)]
pub fn arb_cat_of_units<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, (Index<Pattern>, Vec<NoteUnit>)>> {
    arb_pattern_with_units(Pattern::Cat)
}
