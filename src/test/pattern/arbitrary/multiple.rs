use proptest::prelude::Strategy;
use proptest::prelude::any;
use proptest::prelude::prop;

use crate::arena::Arena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::structures::chain::Chain;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::unit::arb_pattern_note_unit;

const MAX_UNIT_COUNT: usize = 50;

pub fn arb_pattern_with_units<Arenas: PatternArenas + 'static>(
    make_pattern: fn(Multiple<Pattern>) -> Pattern,
) -> impl Strategy<Value = ArenasTo<Arenas, (Index<Pattern>, Vec<NoteUnit>)>> {
    let arenas_to_units = |units: Vec<NoteUnit>| {
        units
            .into_iter()
            .map(arb_pattern_note_unit)
    };
    prop::collection::vec(any::<NoteUnit>(), ..MAX_UNIT_COUNT)
        .prop_map(move |units| (units.clone(), arenas_to_units(units)))
        .prop_map(move |(units, arenas_to_units)| {
            ArenasTo::new(move |arenas: &Arenas| {
                let mut multiple = Multiple::new_empty();
                for unit in arenas_to_units.clone() {
                    let alloc_unit = unit.call(arenas)?;
                    let alloc_chain = arenas
                        .get_pattern_chain_arena()
                        .alloc(Chain(alloc_unit, None, None))?;
                    multiple.push_back(
                        arenas.get_pattern_chain_arena(),
                        alloc_chain,
                    );
                }
                let alloc_pattern = arenas
                    .get_pattern_arena()
                    .alloc(make_pattern(multiple))?;
                Ok((alloc_pattern, units.clone()))
            })
        })
}
