use crate::ast::parser::array::ParserVec;
use crate::ast::pattern::Pattern;
use crate::ast::time::CycleTime;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;

pub mod linked;

pub const MAX_IDENT_LENGTH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier(pub ParserVec<u8, MAX_IDENT_LENGTH>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringLiteral;

#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    /// Function call; e.g. `cat(A5,B5,C5)`.
    /// Formats:
    /// - Non-method: `<ident>((<expr>,)*)`
    /// - Method: `<expr>.<ident>((<expr>,)*)`
    Call {
        is_method: bool,
        name: Identifier,
        args: Multiple<ExprNode>,
        opt_pattern: Option<Pattern>,
    },
    /// Lambda function; e.g. `x=>cat(x,A5)`.
    /// Format: `<ident>=><expr>`
    Lambda { param: Identifier, expr: Option<Index<ExprNode>> },
    /// Array literal; e.g. `[bd,sd,hh]`
    /// Format: `[(<expr>,)*]`
    Array { args: Multiple<ExprNode> },
    /// Expression unit; for details and format see [ExprUnit].
    Unit(ExprUnit),
}

/// Expression unit; can be an identifier e.g. `cat` or `x`; a fixed-point
/// cycle time number e.g. `-5.12`; or a singly-quoted string `'foo'`.
#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprUnit {
    /// Identifier with pattern data attached.
    /// Format: <ident>
    Ident { name: Identifier, as_pattern: Option<Pattern> },
    /// Number represented as a cycle time.
    /// Format: <number>
    Number(CycleTime),
    /// Singly quoted strings; i.e. 'https://...'
    /// Should only be used for URLs.
    /// Stores only the span of the inner portion.
    /// Format: <string>
    String(StringLiteral),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExprNode {
    pub parent: Option<Index<Self>>,
    pub sibling_chain: Chain<Self>,
    pub expr: Expr,
}

impl ExprNode {
    pub const fn new(expr: Expr) -> Self {
        Self { parent: None, sibling_chain: Chain::new(), expr }
    }
}
