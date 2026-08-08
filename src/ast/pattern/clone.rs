use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::clone::clone_linked;
use crate::mem::linked::drop_linked;

#[derive(Debug)]
pub struct PatternCloneDropAdapter<'a, Arenas: PatternArenas>(
    ArenaResult<Option<Index<PatternNode>>>,
    &'a Arenas,
);

impl<'a, Arenas: PatternArenas> PatternCloneDropAdapter<'a, Arenas> {
    pub fn new(index: Index<PatternNode>, arenas: &'a Arenas) -> Self {
        Self(Ok(Some(index)), arenas)
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

impl<'a, Arenas: PatternArenas> Drop for PatternCloneDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.take_opt_item() {
            debug_unwrap(drop_linked(index, self.1));
        }
    }
}

impl<'a, Arenas: PatternArenas> Clone for PatternCloneDropAdapter<'a, Arenas> {
    fn clone(&self) -> Self {
        let cloned_index = match self.0.clone() {
            Ok(Some(index)) => {
                clone_linked(index, self.1, self.1, PatternNode::clone)
                    .map(Some)
            }
            other => other,
        };

        Self(cloned_index, self.1)
    }
}
