use core::cmp::Ordering;

use chumsky::Parser;

use crate::ast::PatternNode;
use crate::ast::parser::Parseable;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;
use crate::test::mem::arena_test::ArenaTest;
use crate::test::mem::arena_test::arenas_test_run;
use crate::test::pattern::arbitrary::strategy::AnyPatternStrategy;

struct ValidatePattern;
impl<Arenas: PatternArenas> ArenaTest<Index<PatternNode>, Arenas>
    for ValidatePattern
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        orig_index: Index<PatternNode>,
    ) {
        let display_adapter =
            PatternDisplayAdapter::new(orig_index.clone(), &shared_arena_ref);
        let orig_formatted = format!("{display_adapter}");
        let parser = PatternDropAdapter::parser(shared_arena_ref.clone());
        let mut result_adapter = parser
            .parse(orig_formatted.as_bytes())
            .into_result()
            .unwrap_or_else(|e| panic!(
                "Parsing pattern {orig_formatted} should succeed, error: {e:?}"
            ))
            .expect("Internal pattern should be correct");
        let result_index = result_adapter
            .take_index()
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
fn can_parse_pattern() {
    arenas_test_run::<
        Index<PatternNode>,
        ValidatePattern,
        GrowableArena<PatternNode>,
        AnyPatternStrategy,
        ArbPatternError,
    >();
}
