use crate::ast::pattern::Pattern;
use crate::ast::time::CycleTime;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringLiteral;

#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternExpr {
    // Non-method: <ident>((<expr>,)*)
    // Method: <expr>.<ident>((<expr>,)*)
    Call {
        is_method: bool,
        name: Index<Identifier>,
        args: Multiple<PatternExprNode>,
        opt_pattern: Option<Pattern>,
    },
    // <ident>=><expr>
    Lambda {
        param: Index<Identifier>,
        expr: Index<PatternExprNode>,
    },
    // Array literal.
    // [(<expr>,)*]
    Array {
        args: Multiple<PatternExprNode>,
    },
    // Unit.
    Unit(ExprUnit),
}

#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprUnit {
    // Identifier with pattern data attached.
    // Format: <ident>
    Ident { name: Index<Identifier>, as_pattern: Option<Pattern> },
    // All numbers are represented as a cycle time.
    // Format: <number>
    Number(CycleTime),
    // Singly quoted strings; i.e. 'https://...'
    // Should only be used for URLs.
    // Stores only the span of the inner portion.
    // Format: <string>
    String(StringLiteral),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatternExprNode {
    pub parent: Option<Index<Self>>,
    pub sibling_chain: Chain<Self>,
    pub expr: PatternExpr,
}

impl PatternExprNode {
    pub const fn new(expr: PatternExpr) -> Self {
        Self { parent: None, sibling_chain: Chain::new(), expr }
    }
}
