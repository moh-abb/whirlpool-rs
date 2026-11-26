use combine::EasyParser;
use proptest::strategy::Strategy;

use crate::ast::expr::ExprNode;
use crate::ast::expr::ident::Identifier;
use crate::mem::ArenaError;
use crate::mem::GrowableArena;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenaTest;
use crate::mem::arena::test::ArenasTo;
use crate::mem::arena::test::StrategyWithArena;
use crate::mem::arena::test::arenas_test_run;

struct AnyIdentStrategy;
impl<Arenas> StrategyWithArena<Identifier, Arenas, ArenaError>
    for AnyIdentStrategy
{
    fn item_strategy<'r>()
    -> impl Strategy<Value = ArenasTo<'r, Arenas, Identifier, ArenaError>>
    where
        Arenas: 'r,
    {
        Identifier::arbitrary()
            .prop_map(|ident| ArenasTo::new(move |_arenas| Ok(ident.clone())))
    }
}

struct ValidateIdent;
impl<Arenas> ArenaTest<Identifier, Arenas> for ValidateIdent {
    fn run<'r>(_arenas: SharedArenaRef<'r, Arenas>, ident: Identifier) {
        let input = ident
            .as_ref()
            .as_slice()
            .expect("arbitrary identifier should not reach capacity");
        let mut parser = Identifier::parser();
        let result = parser
            .easy_parse(input)
            .map_err(|err| err.map_position(|p| p.translate_position(input)))
            .expect("Parsing single CycleTime should succeed")
            .0
            .expect("Parsed CycleTime should not overflow");
        assert_eq!(
            result, ident,
            "Parsed identifier {:?} should be equal to original {:?}",
            result, ident
        );
    }
}

#[test]
fn can_parse_identifier() {
    arenas_test_run::<
        Identifier,
        ValidateIdent,
        GrowableArena<ExprNode>,
        AnyIdentStrategy,
        ArenaError,
    >();
}
