use crate::ast::PatternNode;
use crate::mem::Arena;

pub trait PatternArenas: Arena<PatternNode> {}

impl<A: Arena<PatternNode>> PatternArenas for A {}
