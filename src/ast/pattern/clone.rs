use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::clone_linked;
use crate::mem::linked::drop_linked;

#[derive(Debug)]
pub struct PatternCloneDropAdapter<Arenas: PatternArenas>(
    ArenaResult<Option<Index<PatternNode>>>,
    Arenas,
);

impl<A: PatternArenas> PatternCloneDropAdapter<A> {
    pub fn new(index: Index<PatternNode>, arena: A) -> Self {
        Self(Ok(Some(index)), arena)
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

impl<Arenas: PatternArenas> Drop for PatternCloneDropAdapter<Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.take_opt_item() {
            debug_unwrap(drop_linked(index, &mut self.1));
        }
    }
}

impl<Arenas: PatternArenas + Clone> Clone for PatternCloneDropAdapter<Arenas> {
    fn clone(&self) -> Self {
        let cloned_index = match self.0.clone() {
            Ok(Some(index)) => {
                let mut arena_1 = self.1.clone();
                let mut arena_2 = self.1.clone();
                clone_linked(
                    index,
                    &mut arena_1,
                    &mut arena_2,
                    PatternNode::clone,
                )
                .map(Some)
            }
            other => other,
        };

        Self(cloned_index, self.1.clone())
    }
}
