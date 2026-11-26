#![cfg(test)]

use proptest::collection::vec;
use proptest::prop_oneof;
use proptest::strategy::BoxedStrategy;
use proptest::strategy::Just;
use proptest::strategy::Strategy;
use proptest::string::bytes_regex;

use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::StringLiteral;
use crate::ast::expr::adapter::ExprArenaAdapter;
use crate::ast::expr::ident::Identifier;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::expr::linked::ExprPushBackError;
use crate::ast::parser::array::ParserVec;
use crate::ast::time::arbitrary::arb_cycle_time;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::SharedArenaRef;
use crate::mem::arena::test::ArenasTo;
use crate::mem::linked::LinkedNode;
use crate::mem::linked::checker::check_acyclic;

#[derive(derive_more::From, Debug)]
pub enum ArbExprError {
    ArenaErr(ArenaError),
    ExprPushBackErr(ExprPushBackError),
    ExpectedFullAdapter,
}

impl Identifier {
    const MAX_TAIL_LENGTH: usize = Identifier::MAX_LENGTH - 1;
    /// Allows for characters ^, #, ~ to be part of note literals.
    const HEAD_REGEX: &str = r"[a-zA-Z_~]";
    const TAIL_REGEX: &str = r"[a-zA-Z0-9_~#\^]";

    pub fn arbitrary() -> impl Strategy<Value = Self> {
        let head = bytes_regex(Self::HEAD_REGEX)
            .expect("Regex head creation should succeed");
        let tail = bytes_regex(Self::TAIL_REGEX)
            .expect("Regex head creation should succeed");
        (head, vec(tail, 0..=Self::MAX_TAIL_LENGTH)).prop_map(|(x, xs)| {
            let mut bytes = ParserVec::new();
            bytes.extend(x);
            bytes.extend(xs.into_iter().flatten());
            Self::new(bytes)
        })
    }
}

impl ExprUnit {
    fn arbitrary() -> impl Strategy<Value = Self> {
        prop_oneof![
            Identifier::arbitrary().prop_map(ExprUnit::Ident),
            arb_cycle_time().prop_map(ExprUnit::Number),
            Just(StringLiteral).prop_map(ExprUnit::String),
        ]
    }
}

/// Used as a workaround since `prop_recursive` requires a strategy that has
/// `'static` lifetime, but since we are dealing with values that may borrow
/// arenas with a shorter lifetime, we need to first create the tree with
/// placeholder values and then traverse it again layer-by-layer to get the
/// final tree.
#[derive(Debug, Clone)]
enum PlaceholderTree {
    Call { is_receiver: bool, name: Identifier, args: Vec<Self> },
    Lambda { param: Identifier, expr: Box<Self> },
    Array { args: Vec<Self> },
    Unit(ExprUnit),
}

impl PlaceholderTree {
    fn convert_to_adapter<'r, A: ExprArenas + 'r>(
        &self,
        arena: &mut SharedArenaRef<'r, A>,
    ) -> Result<ExprArenaAdapter<SharedArenaRef<'r, A>>, ArbExprError> {
        let push_to_arenas = |expr: Expr| {
            Result::<_, ArbExprError>::Ok(ExprArenaAdapter::new(
                arena
                    .clone()
                    .push(ExprNode(LinkedNode::new(expr)))?,
                arena.clone(),
            ))
        };

        let push_child =
            |opt_acc: Result<ExprArenaAdapter<_>, ArbExprError>,
             opt_x: Result<ExprArenaAdapter<_>, ArbExprError>| {
                let acc = opt_acc?;
                let acc_index = acc
                    .clone_index()?
                    .expect("Acc adapter should be full");
                arena
                    .clone()
                    .push_child_adapter(acc_index, opt_x?)?;
                Result::<_, ArbExprError>::Ok(acc)
            };

        let push_children = |start: Expr, args: &[Self]| {
            args.iter()
                .map(|arg| arg.convert_to_adapter(&mut arena.clone()))
                .fold(push_to_arenas(start), push_child)
        };

        match self {
            Self::Call { is_receiver, name, args } => push_children(
                Expr::Call {
                    is_receiver: *is_receiver,
                    name: name.clone(),
                    args: Multiple::new(),
                },
                args,
            ),
            Self::Lambda { param, expr } => push_children(
                Expr::Lambda { param: param.clone(), expr: None },
                &[expr.as_ref().clone()],
            ),
            Self::Array { args } => {
                push_children(Expr::Array { args: Multiple::new() }, args)
            }
            Self::Unit(unit) => push_to_arenas(Expr::Unit(unit.clone())),
        }
    }

    fn arbitrary(
        depth: u32,
        max_number_of_nodes: u32,
        items_per_collection: u32,
        max_children_per_node: u32,
    ) -> impl Strategy<Value = Self> {
        let max_children_per_node_usize =
            usize::try_from(max_children_per_node)
                .expect("Converting max number of children per node failed");

        let recurse_tree = move |unit: BoxedStrategy<Self>| {
            // Avoid generating calls with receiver for now.
            let call =
                (
                    prop_oneof![Just(false)],
                    Identifier::arbitrary(),
                    vec(unit.clone(), 0..max_children_per_node_usize),
                )
                    .prop_map(|(is_receiver, name, args)| {
                        Self::Call { is_receiver, name, args }
                    });

            let lambda =
                (Identifier::arbitrary(), unit.clone().prop_map(Box::new))
                    .prop_map(|(param, expr)| Self::Lambda { param, expr });

            let array = vec(unit.clone(), 0..max_children_per_node_usize)
                .prop_map(|args| Self::Array { args });

            prop_oneof![unit, call, lambda, array]
        };

        ExprUnit::arbitrary()
            .prop_map(Self::Unit)
            .prop_recursive(
                depth,
                max_number_of_nodes,
                items_per_collection,
                recurse_tree,
            )
    }
}

pub fn arb_large_expr<'r, Arenas: ExprArenas + 'r>()
-> impl Strategy<Value = ArenasTo<'r, Arenas, Index<ExprNode>, ArbExprError>> {
    PlaceholderTree::arbitrary(8, 256, 10, 10).prop_map(|tree| {
        ArenasTo::new(move |mut arenas| {
            tree.convert_to_adapter(&mut arenas)
                .and_then(|mut adapter| {
                    let index = adapter
                        .take_index()?
                        .ok_or(ArbExprError::ExpectedFullAdapter)?;
                    check_acyclic(index.clone(), &arenas);
                    Ok(index)
                })
        })
    })
}
