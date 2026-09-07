use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::clone::PatternCloneDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::time::CycleTime;
use crate::ast::time::OverflowError;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Cow;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::SharedArenaRef;

pub trait PatternArenas: Arena<PatternNode> {}

impl<A: Arena<PatternNode>> PatternArenas for A {}

/// Represents the error type when pushing back an element to a pattern using
/// [PatternArenasExt].
#[derive(derive_more::From, Debug)]
pub enum PatternPushBackError {
    ArenaErr(ArenaError),
    OverflowErr(OverflowError),
    ExpectedTimedStepPattern,
    ExpectedNormalPattern,
    ExpectedFullAdapter,
}

pub type PatternPushBackResult<T> = Result<T, PatternPushBackError>;

pub trait PatternArenasExt: PatternArenas + Sized {
    fn push_raw(
        &mut self,
        pattern: Pattern,
    ) -> PatternPushBackResult<Index<PatternNode>> {
        self.push(PatternNode::new(pattern))
            .map_err(Into::into)
    }

    fn push_timed_step_adapter(
        &mut self,
        parent_index: Index<PatternNode>,
        time: CycleTime,
        child_adapter: PatternDropAdapter<Self>,
    ) -> PatternPushBackResult<()>;

    fn push_pattern_adapter(
        &mut self,
        parent_index: Index<PatternNode>,
        child_adapter: PatternDropAdapter<Self>,
    ) -> PatternPushBackResult<()>;

    fn push_dropping(
        &mut self,
        pattern: Pattern,
    ) -> PatternPushBackResult<PatternDropAdapter<Self>>;

    fn push_cloning(
        &mut self,
        pattern: Pattern,
    ) -> PatternPushBackResult<PatternCloneDropAdapter<Self>>;
}

impl<'r, A: PatternArenas> PatternArenasExt for SharedArenaRef<'r, A> {
    fn push_dropping(
        &mut self,
        pattern: Pattern,
    ) -> PatternPushBackResult<PatternDropAdapter<Self>> {
        Ok(PatternDropAdapter::new(self.push_raw(pattern)?, self.clone()))
    }

    fn push_cloning(
        &mut self,
        pattern: Pattern,
    ) -> PatternPushBackResult<PatternCloneDropAdapter<Self>> {
        Ok(PatternCloneDropAdapter::new(self.push_raw(pattern)?, self.clone()))
    }

    fn push_timed_step_adapter(
        &mut self,
        parent_index: Index<PatternNode>,
        time: CycleTime,
        child_adapter: PatternDropAdapter<Self>,
    ) -> PatternPushBackResult<()> {
        let timed_step = Pattern::TimedStep(TimedStep(time, Multiple::new()));
        let timed_step_adapter = self.clone().push_dropping(timed_step)?;

        let timed_step_index = timed_step_adapter
            .clone_index()
            .ok_or(PatternPushBackError::ExpectedFullAdapter)?;

        self.push_pattern_adapter(timed_step_index, child_adapter)?;
        self.push_pattern_adapter(parent_index, timed_step_adapter)?;

        Result::<_, PatternPushBackError>::Ok(())
    }

    fn push_pattern_adapter(
        &mut self,
        parent_index: Index<PatternNode>,
        mut child_adapter: PatternDropAdapter<Self>,
    ) -> PatternPushBackResult<()> {
        let child_index = child_adapter
            .clone_index()
            .ok_or(PatternPushBackError::ExpectedFullAdapter)?;

        // Link the pattern to the provided parent.
        let mut shared_arena_ref_1 = self.clone();
        let mut shared_arena_ref_2 = self.clone();
        Multiple::push_back(
            &mut shared_arena_ref_1,
            &mut shared_arena_ref_2,
            parent_index,
            Cow::Indexed(child_index),
        )?;

        // Avoid dropping the newly-linked child
        child_adapter.take_index();

        Ok(())
    }
}

#[derive(derive_more::From, Debug)]
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
