use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::PatternArenasExt;
use crate::ast::pattern::parser::ParsePatternError;
use crate::ast::pattern::parser::ParsePatternResult;
use crate::ast::time::CycleTime;
use crate::mem::Index;
use crate::mem::LinkedContainer;
use crate::mem::SharedArenaRef;
use crate::mem::debug_unwrap;

type ParentAndArena<'src, A> =
    (Result<Index<PatternNode>, ParsePatternError>, SharedArenaRef<'src, A>);

pub struct CollectIntoPattern<'src, A: PatternArenas> {
    opt_parent_and_arena: Option<ParentAndArena<'src, A>>,
    accum_err: Option<ParsePatternError>,
    pushed_count: u16,
}

impl<'src, A: PatternArenas> Default for CollectIntoPattern<'src, A> {
    fn default() -> Self {
        Self { opt_parent_and_arena: None, accum_err: None, pushed_count: 0 }
    }
}

impl<'src, A: PatternArenas> Drop for CollectIntoPattern<'src, A> {
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
            let pop_result = PatternNode::pop_child(
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

impl<'src, A: PatternArenas> CollectIntoPattern<'src, A> {
    fn set_parent_and_arena(
        &mut self,
        parent_and_arena: ParentAndArena<'src, A>,
    ) {
        self.opt_parent_and_arena = Some(parent_and_arena);
    }

    fn add_error(&mut self, error: ParsePatternError) {
        self.accum_err = Some(error);
    }

    pub fn parent_index_of(
        opt_parent: &ParsePatternResult<'src, A>,
    ) -> Result<Index<PatternNode>, ParsePatternError> {
        opt_parent
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|parent| {
                parent
                    .clone_index()
                    .ok_or(ParsePatternError::ExpectedSetParent)
            })
    }

    pub fn into_result(
        &mut self,
        opt_opt_parent: Option<ParsePatternResult<'src, A>>,
    ) -> ParsePatternResult<'src, A> {
        if let Some(error) = self.accum_err.take() {
            return Err(error);
        }
        // Take the parent and arena to avoid dropping the newly-allocated
        // elements after converting into the result, and after we have reached
        // success.
        let _ = self.opt_parent_and_arena.take();
        opt_opt_parent
            .ok_or(ParsePatternError::ExpectedSetParent)
            .flatten()
    }

    fn get_opt_parent_and_arena(
        &self,
    ) -> Result<ParentAndArena<'src, A>, ParsePatternError> {
        self.opt_parent_and_arena
            .clone()
            .ok_or(ParsePatternError::ExpectedSetParent)
    }

    fn push_child_pattern(&mut self, opt_child: ParsePatternResult<'src, A>) {
        let try_push = || {
            let (opt_index, mut arena) = self.get_opt_parent_and_arena()?;
            arena.push_pattern_adapter(opt_index?, opt_child?)?;
            Ok(())
        };

        if let Err(error) = try_push() {
            self.add_error(error);
        }
    }

    fn push_timed_step_pattern(
        &mut self,
        opt_time: Result<CycleTime, ParsePatternError>,
        opt_child: ParsePatternResult<'src, A>,
    ) {
        let try_push = || {
            let (opt_index, mut arena) = self.get_opt_parent_and_arena()?;
            arena.push_timed_step_adapter(opt_index?, opt_time?, opt_child?)?;
            Ok(())
        };

        if let Err(error) = try_push() {
            self.add_error(error);
        }
    }
}

pub struct PatternToCollect<'src, A: PatternArenas> {
    pub parent_and_arena: ParentAndArena<'src, A>,
    pub opt_child: ParsePatternResult<'src, A>,
}

pub struct TimedStepToCollect<'src, A: PatternArenas> {
    pub parent_and_arena: ParentAndArena<'src, A>,
    pub opt_time: Result<CycleTime, ParsePatternError>,
    pub opt_child: ParsePatternResult<'src, A>,
}

impl<'src, A: PatternArenas> Extend<PatternToCollect<'src, A>>
    for CollectIntoPattern<'src, A>
{
    fn extend<T: IntoIterator<Item = PatternToCollect<'src, A>>>(
        &mut self,
        iter: T,
    ) {
        iter.into_iter().for_each(|item| {
            self.set_parent_and_arena(item.parent_and_arena);
            self.push_child_pattern(item.opt_child);
        })
    }
}

impl<'src, A: PatternArenas> Extend<TimedStepToCollect<'src, A>>
    for CollectIntoPattern<'src, A>
{
    fn extend<T: IntoIterator<Item = TimedStepToCollect<'src, A>>>(
        &mut self,
        iter: T,
    ) {
        iter.into_iter().for_each(|item| {
            self.set_parent_and_arena(item.parent_and_arena);
            self.push_timed_step_pattern(item.opt_time, item.opt_child);
        })
    }
}
