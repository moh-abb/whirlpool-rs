use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;
use crate::mem::debug_unwrap;
use crate::mem::linked::clone_linked;
use crate::mem::linked::drop_linked;

#[derive(Debug)]
pub struct PatternCloneDropAdapter<'r, 'a, Arenas: PatternArenas>(
    ArenaResult<Option<Index<PatternNode>>>,
    SharedArenaRef<'r, 'a, PatternNode, Arenas>,
);

impl<'r, 'a, A: PatternArenas> PatternCloneDropAdapter<'r, 'a, A> {
    pub fn new(
        index: Index<PatternNode>,
        arena_ref: SharedArenaRef<'r, 'a, PatternNode, A>,
    ) -> Self {
        Self(Ok(Some(index)), arena_ref)
    }

    pub fn take_opt_item(&mut self) -> Option<Index<PatternNode>> {
        self.0
            .as_mut()
            .ok()
            .map(Option::take)
            .flatten()
    }

    pub fn take_item(&mut self) -> Index<PatternNode> {
        self.take_opt_item().unwrap()
    }
}

impl<'r, 'a, Arenas: PatternArenas> Drop
    for PatternCloneDropAdapter<'r, 'a, Arenas>
{
    fn drop(&mut self) {
        if let Some(index) = self.take_opt_item() {
            debug_unwrap(drop_linked(index, &mut self.1));
        }
    }
}

impl<'r, 'a, Arenas: PatternArenas> Clone
    for PatternCloneDropAdapter<'r, 'a, Arenas>
{
    fn clone(&self) -> Self {
        let cloned_index = match self.0.clone() {
            Ok(Some(index)) => {
                let mut shared_ref_2 = self.1.clone();
                let mut shared_ref_1 = self.1.clone();
                clone_linked(
                    index,
                    &mut shared_ref_1,
                    &mut shared_ref_2,
                    PatternNode::clone,
                )
                .map(Some)
            }
            other => other,
        };

        Self(cloned_index, self.1.clone())
    }
}
