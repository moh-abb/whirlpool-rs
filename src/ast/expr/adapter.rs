use crate::ast::expr::ExprNode;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::debug_unwrap;
use crate::mem::linked::clone::clone_linked;
use crate::mem::linked::drop::drop_linked;

#[derive(Debug)]
pub struct ExprArenaAdapter<Arenas: ExprArenas> {
    result: ArenaResult<Option<Index<ExprNode>>>,
    arenas: Arenas,
}

impl<Arenas: ExprArenas> ExprArenaAdapter<Arenas> {
    pub fn new(index: Index<ExprNode>, arenas: Arenas) -> Self {
        Self { result: Ok(Some(index.clone())), arenas }
    }

    pub fn clone_index(&self) -> ArenaResult<Option<Index<ExprNode>>> {
        self.result.clone()
    }

    pub fn take_index(&mut self) -> ArenaResult<Option<Index<ExprNode>>> {
        self.result
            .as_mut()
            .map(Option::take)
            .map_err(|error| error.clone())
    }
}

impl<Arenas: Clone + ExprArenas> Clone for ExprArenaAdapter<Arenas> {
    fn clone(&self) -> Self {
        let clone_result = match self.result.clone() {
            Ok(Some(index)) => {
                let arena_1 = self.arenas.clone();
                let mut arena_2 = self.arenas.clone();
                // We assume that everything outside the `ExprNode` can be
                // cloned only using `Clone`.
                clone_linked(index, &arena_1, &mut arena_2, ExprNode::clone)
                    .map(Some)
            }
            other => other,
        };

        Self { result: clone_result, arenas: self.arenas.clone() }
    }
}

impl<Arenas: ExprArenas> Drop for ExprArenaAdapter<Arenas> {
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
