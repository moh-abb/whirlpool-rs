//! Arbitrary test for the custom recursive combinator, using a binary
//! expression parser.

use core::fmt;

use combine::EasyParser;
use combine::Parser;
use combine::between;
use combine::chainl1;
use combine::choice;
use combine::easy::ParseError;
use combine::parser;
use combine::token;
use proptest::arbitrary::any;
use proptest::collection::vec;
use proptest::strategy::Strategy;
use proptest::test_runner::TestRunner;

use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::recursive::arena_recursive;
use crate::ast::parser::recursive::error::RecResult;
use crate::mem::GrowableArena;
use crate::mem::SharedArena;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum LogicExpr {
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Unit(bool),
}

use LogicExpr::*;

parser! {
    fn normal_expr_parser[I]()(I) -> RecResult<LogicExpr>
    where
        [I: AstInput]
    {
        LogicExpr::rec_parser(|| normal_expr_parser())
    }
}

impl fmt::Display for LogicExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Should we add disambiguating brackets for precedence?
        let should_add_brackets =
            |parent: &Self, child: &Self| match (parent, child) {
                // When the expressions are the same, add brackets to determine
                // associativity.
                (And(_, _), And(_, _)) => true,
                (Or(_, _), Or(_, _)) => true,
                (And(_, _), Or(_, _)) => true,
                // We don't need to add brackets to the child expression if it
                // uses a stronger operator.
                (Or(_, _), And(_, _)) => false,
                (_, Unit(_)) | (Unit(_), _) => false,
            };
        let expr_symbol = |parent: &Self| match parent {
            And(_, _) => "^",
            Or(_, _) => "|",
            Unit(_) => "",
        };
        match self {
            Unit(true) => write!(f, "1"),
            Unit(false) => write!(f, "0"),
            Or(x, y) | And(x, y) => {
                [x, y]
                    .iter()
                    .enumerate()
                    .try_for_each(|(i, expr)| {
                        if i != 0 {
                            write!(f, "{}", expr_symbol(self))?;
                        }
                        if should_add_brackets(self, expr.as_ref()) {
                            write!(f, "({expr})")
                        } else {
                            write!(f, "{expr}")
                        }
                    })
            }
        }
    }
}

impl LogicExpr {
    fn make_expr(
        f: impl Fn(Box<Self>, Box<Self>) -> Self,
        mut xs: Vec<Self>,
    ) -> Self {
        if xs.len() == 1 {
            // If there is only one subexpression, return it directly.
            let Some(x) = xs.pop() else {
                panic!("Expression should have at least one value");
            };
            x
        } else {
            xs.into_iter()
                .reduce(|x, y| f(Box::new(x), Box::new(y)))
                .expect("make_expr: requires nonempty vector xs")
        }
    }

    fn arbitrary() -> impl Strategy<Value = Self> {
        let depth = 8;
        let desired_size = 256;
        let branch_size = 6;
        let max_vec_size = 10;

        any::<bool>()
            .prop_map(Self::Unit)
            .prop_recursive(depth, desired_size, branch_size, move |inner| {
                let func = any::<bool>()
                    .prop_map(|chosen| if chosen { Or } else { And });
                (func, vec(inner, 1..max_vec_size))
                    .prop_map(|(f, xs)| Self::make_expr(f, xs))
            })
    }

    fn rec_parser<I, F, RecCb>(
        rec_callback: F,
    ) -> impl AstParser<I, RecResult<Self>>
    where
        I: AstInput,
        RecCb: AstParser<I, RecResult<Self>>,
        F: Fn() -> RecCb,
    {
        let unit_bool = choice(
            [b'0', b'1']
                .map(|b| token(b).map(move |_| Ok(Self::Unit(b != b'0')))),
        );
        let unit_rec = between(token(b'('), token(b')'), rec_callback());
        let logic_unit = unit_bool.or(unit_rec);

        let logic_and = chainl1(
            logic_unit,
            token(b'^')
                .map(|_| |x, y| Ok(Self::And(Box::new(x?), Box::new(y?)))),
        );

        let logic_or = chainl1(
            logic_and,
            token(b'|')
                .map(|_| |x, y| Ok(Self::Or(Box::new(x?), Box::new(y?)))),
        );

        logic_or
    }
}

type ExprParseResult<'a> =
    Result<(RecResult<LogicExpr>, &'a [u8]), ParseError<&'a [u8]>>;

fn verify_parse_result(
    parse_result: ExprParseResult<'_>,
    input: &[u8],
    expected: LogicExpr,
) {
    let (opt_parsed, remainder) = parse_result.unwrap_or_else(|error| {
        panic!(
            "Parsing {} gave parser error {:?}",
            String::from_utf8_lossy(input),
            error,
        )
    });
    let parsed = opt_parsed.unwrap_or_else(|error| {
        panic!(
            "Parsing {} gave recursive result with error {:?}",
            String::from_utf8_lossy(input),
            error,
        )
    });
    assert_eq!(
        parsed,
        expected,
        "Parsed value {parsed} not equal to {expected} (original text {:?})",
        String::from_utf8_lossy(input),
    );
    assert!(
        remainder.is_empty(),
        "Parsing {} gave remainder {} which is not empty",
        String::from_utf8_lossy(input),
        String::from_utf8_lossy(remainder),
    );
}

fn parse_arbitrary<F>(run_parse: F)
where
    F: for<'a> Fn(&'a [u8]) -> ExprParseResult<'a>,
{
    let mut test_runner = TestRunner::deterministic();
    let strat = LogicExpr::arbitrary();
    test_runner
        .run(&strat, move |expected| {
            let input = format!("{expected}").into_bytes();
            verify_parse_result(run_parse(&input), &input, expected);
            Ok(())
        })
        .unwrap()
}

#[test]
fn can_parse_arbitrary_logic_expr_normally() {
    parse_arbitrary(|input| normal_expr_parser().easy_parse(input))
}

#[test]
fn can_parse_arbitrary_logic_expr_with_arena() {
    parse_arbitrary(|input| {
        let output_arena = SharedArena::new(GrowableArena::new());
        let frame_arena = SharedArena::new(GrowableArena::new());

        let output_arena_ref = output_arena.make_ref();
        let frame_arena_ref = frame_arena.make_ref();
        let mut arena_expr_parser = arena_recursive(
            |cb| LogicExpr::rec_parser(move || cb.clone()),
            output_arena_ref,
            frame_arena_ref,
            1000,
        );

        arena_expr_parser.easy_parse(input)
    })
}
