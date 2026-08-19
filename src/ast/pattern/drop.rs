use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Index;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;
use crate::mem::debug_unwrap;
use crate::mem::linked::drop::drop_linked;

pub struct PatternDropAdapter<'r, 'a, Arenas: PatternArenas>(
    pub Option<Index<PatternNode>>,
    pub SharedArenaRef<'r, 'a, PatternNode, Arenas>,
);

impl<'r, 'a, Arenas: PatternArenas> Drop
    for PatternDropAdapter<'r, 'a, Arenas>
{
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        debug_unwrap(drop_linked(inner, &mut self.1))
    }
}
