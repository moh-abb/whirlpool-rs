use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::clone::clone_linked;
use crate::mem::linked::drop::drop_linked;

pub struct PatternArenaAdapter<Arenas: PatternArenas> {
    result: ArenaResult<Option<Index<PatternNode>>>,
    arenas: Arenas,
}

impl<Arenas: PatternArenas> PatternArenaAdapter<Arenas> {
    pub fn new(index: Index<PatternNode>, arenas: Arenas) -> Self {
        Self { result: Ok(Some(index.clone())), arenas }
    }

    pub fn clone_index(&self) -> ArenaResult<Option<Index<PatternNode>>> {
        self.result.clone()
    }

    pub fn take_index(&mut self) -> ArenaResult<Option<Index<PatternNode>>> {
        self.result
            .as_mut()
            .map(Option::take)
            .map_err(|error| error.clone())
    }
}

impl<Arenas: Clone + PatternArenas> Clone for PatternArenaAdapter<Arenas> {
    fn clone(&self) -> Self {
        let clone_result = match self.result.clone() {
            Ok(Some(index)) => {
                let arena_1 = self.arenas.clone();
                let mut arena_2 = self.arenas.clone();
                // We assume that everything outside the `PatternNode` can be
                // cloned only using `Clone`.
                clone_linked(index, &arena_1, &mut arena_2, PatternNode::clone)
                    .map(Some)
            }
            other => other,
        };

        Self { result: clone_result, arenas: self.arenas.clone() }
    }
}

impl<Arenas: PatternArenas> Drop for PatternArenaAdapter<Arenas> {
    fn drop(&mut self) {
        match &mut self.result {
            Ok(Some(index)) => {
                debug_unwrap(drop_linked(index.clone(), &mut self.arenas))
            }
            Ok(None) => (),
            Err(error) => debug_unwrap(Err(error)),
        }
    }
}
