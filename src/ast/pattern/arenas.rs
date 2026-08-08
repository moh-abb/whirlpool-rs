use core::fmt::Debug;

use crate::ast::PatternNode;
use crate::mem::Arena;

pub trait PatternArenas: Debug {
    fn get_pattern_arena(&self) -> &impl Arena<PatternNode>;
}
