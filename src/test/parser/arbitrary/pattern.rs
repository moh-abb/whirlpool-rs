use core::cmp::Ordering;
use core::fmt::Debug;

use combine::EasyParser;

use crate::ast::pattern::PatternNode;
use crate::ast::pattern::adapter::PatternArenaAdapter;
use crate::ast::pattern::arbitrary::ArbPatternError;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::cmp::PatternOrdAdapter;
use crate::ast::pattern::format::PatternDisplayAdapter;
use crate::mem::GrowableArena;
use crate::mem::Index;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::arenas_test_run;
use crate::test::ast::arbitrary::strategy::AnyPatternStrategy;

struct ValidatePattern;
impl<Arenas: PatternArenas + Debug> ArenaTest<Index<PatternNode>, Arenas>
    for ValidatePattern
{
    fn run<'r>(
        shared_arena_ref: SharedArenaRef<'r, Arenas>,
        orig_index: Index<PatternNode>,
    ) {
        let display_adapter =
            PatternDisplayAdapter::new(orig_index.clone(), &shared_arena_ref);
        let orig_formatted = format!("{display_adapter}");
        let mut parser = PatternArenaAdapter::parser(shared_arena_ref.clone());
        let mut result_adapter = parser
            .easy_parse(orig_formatted.as_bytes())
            .unwrap_or_else(|e| {
                panic!(
                    "Parsing pattern {orig_formatted} \
                    failed with error {e:?}"
                )
            })
            .0
            .unwrap_or_else(|e| {
                panic!(
                    "Parsed pattern {orig_formatted} \
                    contains error {e:?}"
                )
            });
        let result_index = result_adapter
            .take_index()
            .expect("Adapter should not store an error")
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
