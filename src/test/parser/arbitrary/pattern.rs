use core::cmp::Ordering;

use chumsky::Parser;

use crate::ast::PatternNode;
use crate::ast::parser::Parseable;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::arena::arena_impl::shared_arena::SharedArena;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::with_regenerated_arenas;
use crate::test::mem::arena_test::with_reused_arenas;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;

struct ValidatePattern;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for ValidatePattern
{
    fn run(arenas: &mut Arenas, orig_index: Index<PatternNode>) {
        let display_adapter =
            PatternDisplayAdapter::new(orig_index.clone(), arenas);
        let orig_formatted = format!("{display_adapter}");
        let shared_arena = SharedArena::new(arenas);
        let shared_arena_ref = shared_arena.make_ref();
        let parser = PatternDropAdapter::parser(shared_arena_ref);
        let mut result_adapter = parser
            .parse(orig_formatted.as_bytes())
            .into_result()
            .unwrap_or_else(|e| panic!(
                "Parsing pattern {orig_formatted} should succeed, error: {e:?}"
            ))
            .expect("Internal pattern should be correct");
        let result_index = result_adapter
            .0
            .take()
            .expect("Should have a full item inside the adapter");
        let result_formatted =
            PatternDisplayAdapter::new(result_index.clone(), &shared_arena_ref);

        assert_eq!(
            PatternOrdAdapter::new_left(orig_index, &shared_arena_ref).cmp(
                &PatternOrdAdapter::new_right(result_index, &shared_arena_ref)
            ),
            Ordering::Equal,
            "Parsed pattern {result_formatted} should be
            equal to original {orig_formatted}"
        );
    }
}

#[test]
fn can_parse_cycle_time_once() {
    with_regenerated_arenas::<
        Index<PatternNode>,
        ValidatePattern,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >();
}

#[test]
fn can_parse_cycle_time_multiple() {
    with_reused_arenas::<
        Index<PatternNode>,
        ValidatePattern,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
    >();
}
