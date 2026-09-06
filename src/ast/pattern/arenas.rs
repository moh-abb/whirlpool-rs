use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::time::OverflowError;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;

pub trait PatternArenas: Arena<PatternNode> {}

impl<A: Arena<PatternNode>> PatternArenas for A {}

pub trait PatternArenasExt: PatternArenas + Sized {
    fn push_raw(
        &mut self,
        pattern: Pattern,
    ) -> ArenaResult<Index<PatternNode>> {
        self.push(PatternNode::new(pattern))
    }

    fn push_dropping(
        &mut self,
        pattern: Pattern,
    ) -> ArenaResult<PatternDropAdapter<Self>>;

    fn push_cloning(
        &mut self,
        pattern: Pattern,
    ) -> ArenaResult<PatternCloneDropAdapter<Self>>;
}

impl<'r, A: PatternArenas> PatternArenasExt
    for SharedArenaRef<'r, PatternNode, A>
{
    fn push_dropping(
        &mut self,
        pattern: Pattern,
    ) -> ArenaResult<PatternDropAdapter<Self>> {
        Ok(PatternDropAdapter::new(self.push_raw(pattern)?, self.clone()))
    }

    fn push_cloning(
        &mut self,
        pattern: Pattern,
    ) -> ArenaResult<PatternCloneDropAdapter<Self>> {
        Ok(PatternCloneDropAdapter::new(self.push_raw(pattern)?, self.clone()))
    }
}

#[derive(derive_more::From, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SumCycleLengthError {
    ArenaErr(ArenaError),
    OverflowErr(OverflowError),
    ExpectedTimedStep,
}

pub fn sum_cycle_length<T, E: From<OverflowError>>(
    mut iter: impl Iterator<Item = Result<T, E>>,
    mut f: impl FnMut(T) -> CycleTime,
) -> Result<CycleTime, E> {
    iter.try_fold(CycleTime::ZERO, |acc, x| {
        let time = f(x?);
        Ok(acc.add(time)?)
    })
}

pub fn timed_step_total_cycle_length(
    multiple: Multiple<PatternNode>,
    arenas: &impl PatternArenas,
) -> Result<CycleTime, SumCycleLengthError> {
    sum_cycle_length(
        multiple
            .checked_iter(arenas)
            .map(|opt_node_index| {
                let node = arenas.map(opt_node_index?, Clone::clone)?;
                let Pattern::TimedStep(timed_step) = node.pattern else {
                    return Err(SumCycleLengthError::ExpectedTimedStep);
                };
                Ok(timed_step)
            }),
        |TimedStep(dur, _)| dur,
    )
}
