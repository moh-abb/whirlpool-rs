use proptest::prop_oneof;
use proptest::strategy::Just;
use proptest::strategy::Strategy;

use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::arbitrary::arb_note_unit;
use crate::mem::Index;
use crate::test::mem::arenas_to::ArenasTo;

pub fn arb_pattern_leaf<Arenas: PatternArenas + 'static>()
-> impl Strategy<Value = ArenasTo<Arenas, Index<PatternNode>>> {
    let value_strategy = prop_oneof![
        Just(PatternNode::new(Pattern::Silence)),
        arb_note_unit()
            .prop_map(Pattern::Note)
            .prop_map(PatternNode::new)
    ];
    value_strategy.prop_map(move |value| {
        ArenasTo::new(move |arenas: &mut Arenas| arenas.push(value.clone()))
    })
}
