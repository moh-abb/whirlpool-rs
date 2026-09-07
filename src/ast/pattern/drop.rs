use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::drop::drop_linked;

pub struct PatternDropAdapter<Arenas: PatternArenas>(
    Option<Index<PatternNode>>,
    Arenas,
);

impl<Arenas: PatternArenas> PatternDropAdapter<Arenas> {
    pub fn new(index: Index<PatternNode>, arenas: Arenas) -> Self {
        Self(Some(index), arenas)
    }

    pub fn clone_index(&self) -> Option<Index<PatternNode>> {
        self.0.clone()
    }

    pub fn take_index(&mut self) -> Option<Index<PatternNode>> {
        self.0.take()
    }
}

impl<Arenas: PatternArenas> Drop for PatternDropAdapter<Arenas> {
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        debug_unwrap(drop_linked(inner, &mut self.1))
    }
}
