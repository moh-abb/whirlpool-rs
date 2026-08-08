use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::drop::drop_linked;

pub struct PatternDropAdapter<'a, Arenas: PatternArenas>(
    pub Option<Index<PatternNode>>,
    pub &'a Arenas,
);

impl<'a, Arenas: PatternArenas> Drop for PatternDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        debug_unwrap(drop_linked(inner, self.1))
    }
}
