use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::time::OverflowError;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Multiple;

pub trait PatternArenas: Arena<PatternNode> {}

impl<A: Arena<PatternNode>> PatternArenas for A {}

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
