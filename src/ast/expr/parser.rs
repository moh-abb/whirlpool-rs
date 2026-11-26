use core::fmt::Debug;

use combine::Parser;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::many;
use combine::parser;
use combine::satisfy;
use combine::sep_by;
use combine::token;
use combine::tokens_cmp;

use crate::ast::expr::Expr;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::StringLiteral;
use crate::ast::expr::adapter::ExprArenaAdapter;
use crate::ast::expr::ident::Identifier;
use crate::ast::expr::ident::ParseIdentError;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::expr::linked::ExprPushBackError;
use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::collect::CollectIntoExpr;
use crate::ast::parser::collect::ExprToCollect;
use crate::ast::time::CycleTime;
use crate::ast::time::parser::ParseCycleTimeError;
use crate::mem::ArenaError;
use crate::mem::Multiple;
use crate::mem::SharedArenaRef;

#[derive(derive_more::From, Debug, Clone)]
pub enum ParseExprError {
    ExprPushBackErr(ExprPushBackError),
    ParseIdentErr(ParseIdentError),
    ParseCycleTimeErr(ParseCycleTimeError),
    ArenaErr(ArenaError),
    ExpectedSetParent,
}

type ParseResult<T> = Result<T, ParseExprError>;
type ParsedExpr<'src, A> = ExprArenaAdapter<SharedArenaRef<'src, A>>;

pub type ParseExprResult<'src, A> = ParseResult<ParsedExpr<'src, A>>;

fn whitespace<'src, I: AstInput>() -> impl AstParser<I, ()> {
    many::<(), _, _>(token(b' ').map(|_| ()))
}

fn separator<'src, I: AstInput>() -> impl AstParser<I, ()> {
    between(whitespace(), whitespace(), token(b',')).map(|_| ())
}

fn unit<'src, A, I>(
    shared_arena_ref: SharedArenaRef<'src, A>,
) -> impl AstParser<I, ParseExprResult<'src, A>>
where
    A: ExprArenas + Debug,
    I: AstInput,
{
    let leaf_number = CycleTime::parser().map(|opt_time| {
        opt_time
            .map(ExprUnit::Number)
            .map_err(Into::into)
    });
    let leaf_ident = Identifier::parser().map(|opt_ident| {
        opt_ident
            .map(ExprUnit::Ident)
            .map_err(Into::into)
    });
    let leaf_string_literal = between(
        token(b'"'),
        token(b'"'),
        many::<(), _, _>(satisfy(|b: u8| b != b'"').map(|_| ())),
    )
    .map(|()| Ok(ExprUnit::String(StringLiteral)))
    .expected("stringliteral");

    let push_leaf = move |opt_leaf: ParseResult<ExprUnit>| {
        opt_leaf.and_then(|leaf| {
            shared_arena_ref
                .clone()
                .push_to_adapter(Expr::Unit(leaf))
                .map_err(Into::into)
        })
    };

    choice((leaf_string_literal, leaf_ident, leaf_number)).map(push_leaf)
}

fn array<'src, A, I, RP, F>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    mut subexpr: F,
) -> impl AstParser<I, ParseExprResult<'src, A>>
where
    A: ExprArenas + Debug,
    I: AstInput,
    RP: AstParser<I, ParseExprResult<'src, A>>,
    F: FnMut() -> RP + Clone,
{
    let array_start = token(b'[');
    let array_end = token(b']');

    let cloned_arena_ref = shared_arena_ref.clone();
    let array_start_with_adapter = array_start.map(move |_| {
        cloned_arena_ref
            .clone()
            .push_to_adapter(Expr::Array { args: Multiple::new() })
            .map_err(Into::into)
    });

    let cloned_arena_ref_2 = shared_arena_ref.clone();
    array_start_with_adapter
        .then(move |opt_parent| {
            let opt_parent_index =
                CollectIntoExpr::parent_index_of(&opt_parent);
            let parent_and_arena =
                (opt_parent_index.clone(), cloned_arena_ref_2.clone());
            let subexpr_with_parent =
                subexpr().map(move |opt_child| ExprToCollect {
                    parent_and_arena: parent_and_arena.clone(),
                    opt_child,
                });

            let array_elems = sep_by::<CollectIntoExpr<_>, _, _, _>(
                subexpr_with_parent,
                separator(),
            );

            let mut opt_opt_parent = Some(opt_parent);
            (array_elems, array_end.clone()).map(
                move |(mut array_result, _)| {
                    array_result.into_result(opt_opt_parent.take())
                },
            )
        })
        .expected("array")
}

fn call<'src, A, I, RP, F>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    mut subexpr: F,
) -> impl AstParser<I, ParseExprResult<'src, A>>
where
    A: ExprArenas + Debug,
    I: AstInput,
    RP: AstParser<I, ParseExprResult<'src, A>>,
    F: FnMut() -> RP + Clone,
{
    let func_start = Identifier::parser().expected("callname");

    let cloned_arena_ref = shared_arena_ref.clone();
    let func_start_with_adapter = func_start.map(move |opt_name| {
        cloned_arena_ref
            .clone()
            .push_to_adapter(Expr::Call {
                is_receiver: false,
                name: opt_name?,
                args: Multiple::new(),
            })
            .map_err(Into::into)
    });

    func_start_with_adapter
        .then(move |opt_parent| {
            let opt_parent_index =
                CollectIntoExpr::parent_index_of(&opt_parent);
            let parent_and_arena =
                (opt_parent_index.clone(), shared_arena_ref.clone());
            let subexpr_with_parent =
                subexpr().map(move |opt_child| ExprToCollect {
                    parent_and_arena: parent_and_arena.clone(),
                    opt_child,
                });

            let mut opt_opt_parent = Some(opt_parent);
            between(
                token(b'('),
                token(b')'),
                sep_by::<CollectIntoExpr<_>, _, _, _>(
                    subexpr_with_parent,
                    separator(),
                )
                .map(move |mut collected| {
                    collected.into_result(opt_opt_parent.take())
                }),
            )
        })
        .expected("call")
}

fn just_slice<I: AstInput>(slice: &'static [u8]) -> impl AstParser<I, ()> {
    tokens_cmp(slice, |cx: &u8, cy: u8| *cx == cy).map(|_| ())
}

fn lambda<'src, A, I, RP, F>(
    mut shared_arena_ref: SharedArenaRef<'src, A>,
    mut subexpr: F,
) -> impl AstParser<I, ParseExprResult<'src, A>>
where
    A: ExprArenas + Debug,
    I: AstInput,
    RP: AstParser<I, ParseExprResult<'src, A>>,
    F: FnMut() -> RP + Clone,
{
    ((Identifier::parser(), just_slice(b"=>"), subexpr()))
        .map(move |(opt_param, _, opt_expr)| {
            let lambda = shared_arena_ref.push_to_adapter(Expr::Lambda {
                param: opt_param?,
                expr: None,
            })?;
            let parent_index = lambda
                .clone_index()?
                .ok_or(ExprPushBackError::ExpectedFullAdapter)?;
            shared_arena_ref.push_child_adapter(parent_index, opt_expr?)?;
            ParseExprResult::Ok(lambda)
        })
        .expected("lambda")
}

fn choice_expr<'src, A, I, RP, F>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    subexpr: F,
) -> impl AstParser<I, ParseExprResult<'src, A>>
where
    A: ExprArenas + Debug,
    I: AstInput,
    RP: AstParser<I, ParseExprResult<'src, A>>,
    F: FnMut() -> RP + Clone,
{
    // Due to ambiguity in both `call`, `lambda` and `unit` starting with an
    // identifier, avoid committing on either of those parses.
    choice((
        array(shared_arena_ref.clone(), subexpr.clone()),
        attempt(call(shared_arena_ref.clone(), subexpr.clone())),
        attempt(lambda(shared_arena_ref.clone(), subexpr.clone())),
        unit(shared_arena_ref.clone()),
    ))
}

parser! {
    fn recursive_expr['src, A, I](
        shared_arena_ref: SharedArenaRef<'src, A>
    )(I) -> ParseExprResult<'src, A>
    where [
        I: AstInput,
        A: ExprArenas + Debug,
    ]
    {
        choice_expr(
            shared_arena_ref.clone(),
            || recursive_expr(shared_arena_ref.clone()),
        )
    }
}

impl<'src, A: ExprArenas + Debug> ParsedExpr<'src, A> {
    pub fn parser<I: AstInput>(
        shared_arena_ref: SharedArenaRef<'src, A>,
    ) -> impl AstParser<I, Result<Self, ParseExprError>> {
        recursive_expr(shared_arena_ref)
    }
}
