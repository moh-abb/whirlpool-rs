use crate::ast::expr::ExprNode;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::expr::parser::ParseExprError;
use crate::ast::expr::parser::ParseExprResult;
use crate::mem::Index;
use crate::mem::LinkedContainer;
use crate::mem::SharedArenaRef;
use crate::mem::debug_unwrap;

type ParentAndArena<'src, A> =
    (Result<Index<ExprNode>, ParseExprError>, SharedArenaRef<'src, A>);

pub struct CollectIntoExpr<'src, A: ExprArenas> {
    opt_parent_and_arena: Option<ParentAndArena<'src, A>>,
    accum_err: Option<ParseExprError>,
    pushed_count: u16,
}

impl<'src, A: ExprArenas> Default for CollectIntoExpr<'src, A> {
    fn default() -> Self {
        Self { opt_parent_and_arena: None, accum_err: None, pushed_count: 0 }
    }
}

impl<'src, A: ExprArenas> Drop for CollectIntoExpr<'src, A> {
    fn drop(&mut self) {
        let Some((opt_index, arena)) = self.opt_parent_and_arena.take() else {
            return;
        };
        let parent_index = match opt_index {
            Ok(index) => index,
            Err(error) => {
                debug_unwrap(Err(error));
                return;
            }
        };
        while let Some(next_count) = self.pushed_count.checked_sub(1) {
            let pop_result = LinkedContainer::pop_child(
                &mut arena.clone(),
                &mut arena.clone(),
                parent_index.clone(),
                true,
            );
            debug_unwrap(pop_result.map(|_| ()));
            self.pushed_count = next_count;
        }
    }
}

impl<'src, A: ExprArenas> CollectIntoExpr<'src, A> {
    fn set_parent_and_arena(
        &mut self,
        parent_and_arena: ParentAndArena<'src, A>,
    ) {
        self.opt_parent_and_arena = Some(parent_and_arena);
    }

    fn add_error(&mut self, error: ParseExprError) {
        self.accum_err = Some(error);
    }

    pub fn parent_index_of(
        opt_parent: &ParseExprResult<'src, A>,
    ) -> Result<Index<ExprNode>, ParseExprError> {
        opt_parent
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|parent| {
                parent
                    .clone_index()?
                    .ok_or(ParseExprError::ExpectedSetParent)
            })
    }

    pub fn into_result(
        &mut self,
        opt_opt_parent: Option<ParseExprResult<'src, A>>,
    ) -> ParseExprResult<'src, A> {
        if let Some(error) = self.accum_err.take() {
            return Err(error);
        }
        // Take the parent and arena to avoid dropping the newly-allocated
        // elements after converting into the result, and after we have reached
        // success.
        let _ = self.opt_parent_and_arena.take();
        opt_opt_parent
            .ok_or(ParseExprError::ExpectedSetParent)
            .flatten()
    }

    fn get_opt_parent_and_arena(
        &self,
    ) -> Result<ParentAndArena<'src, A>, ParseExprError> {
        self.opt_parent_and_arena
            .clone()
            .ok_or(ParseExprError::ExpectedSetParent)
    }

    fn push_child_expr(&mut self, opt_child: ParseExprResult<'src, A>) {
        let try_push = || {
            let (opt_index, mut arena) = self.get_opt_parent_and_arena()?;
            arena.push_child_adapter(opt_index?, opt_child?)?;
            Ok(())
        };

        if let Err(error) = try_push() {
            self.add_error(error);
        }
    }
}

pub struct ExprToCollect<'src, A: ExprArenas> {
    pub parent_and_arena: ParentAndArena<'src, A>,
    pub opt_child: ParseExprResult<'src, A>,
}

impl<'src, A: ExprArenas> Extend<ExprToCollect<'src, A>>
    for CollectIntoExpr<'src, A>
{
    fn extend<T: IntoIterator<Item = ExprToCollect<'src, A>>>(
        &mut self,
        iter: T,
    ) {
        iter.into_iter().for_each(|item| {
            self.set_parent_and_arena(item.parent_and_arena);
            self.push_child_expr(item.opt_child);
        })
    }
}
