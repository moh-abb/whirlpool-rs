use core::iter::once;

use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::refs::PatternDropRef;
use crate::ast::pattern::refs::combine_iters;
use crate::ast::pattern::refs::pattern_refs;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::debug_unwrap;
use crate::mem::drop::drop_in_arenas;
use crate::mem::drop::drop_in_arenas_from_start_ref;
use crate::mem::drop::refs::DropRefs;

pub struct PatternDropAdapter<'a, Arenas: PatternArenas>(
    pub Option<Index<Pattern>>,
    pub &'a Arenas,
);

pub struct MultiplePatternDropAdapter<'a, Arenas: PatternArenas>(
    pub Option<Multiple<Pattern>>,
    pub &'a Arenas,
);

pub struct TimedStepDropAdapter<'a, Arenas: PatternArenas>(
    pub Option<Index<TimedStep>>,
    pub &'a Arenas,
);

pub struct MultipleTimedStepDropAdapter<'a, Arenas: PatternArenas>(
    pub Option<Multiple<TimedStep>>,
    pub &'a Arenas,
);

impl<'a, Arenas: PatternArenas> Drop for PatternDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        debug_unwrap(drop_in_arenas(inner, self.1))
    }
}

impl<'a, Arenas: PatternArenas> Drop
    for MultiplePatternDropAdapter<'a, Arenas>
{
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        multiple_drop(
            inner,
            self.1.get_pattern_chain_arena(),
            self.1,
            PatternDropRef::Pattern,
        );
    }
}

impl<'a, Arenas: PatternArenas> Drop for TimedStepDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        debug_unwrap(drop_in_arenas_from_start_ref::<Index<Pattern>, _>(
            PatternDropRef::TimedStep(inner),
            self.1,
        ))
    }
}

impl<'a, Arenas: PatternArenas> Drop
    for MultipleTimedStepDropAdapter<'a, Arenas>
{
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };
        multiple_drop(
            inner,
            self.1.get_timed_step_chain_arena(),
            self.1,
            PatternDropRef::TimedStep,
        )
    }
}

/// Helper function to drop a [Multiple].
fn multiple_drop<Item: ArenaItem, Arenas: PatternArenas>(
    multiple: Multiple<Item>,
    chain_arena: &impl Arena<Chain<Item>>,
    arenas: &Arenas,
    make_drop_ref: impl Fn(Index<Item>) -> PatternDropRef,
) {
    let drop_result = multiple
        .iter(chain_arena)
        .try_for_each(|item| {
            drop_in_arenas_from_start_ref::<Index<Pattern>, _>(
                make_drop_ref(item),
                arenas,
            )
        });
    debug_unwrap(drop_result)
}

impl<Arenas: PatternArenas> DropRefs<Arenas> for Index<Pattern> {
    type DropRefType = PatternDropRef;

    fn start_ref(index: Index<Pattern>) -> Self::DropRefType {
        PatternDropRef::Pattern(index)
    }

    fn process_ref<'a>(
        reference: Self::DropRefType,
        arenas: &'a Arenas,
    ) -> ArenaResult<impl Iterator<Item = ArenaResult<Self::DropRefType>> + 'a>
    {
        let pattern_index_to_iter = |index: Index<Pattern>| {
            once(index)
                .map(PatternDropRef::Pattern)
                .map(Ok)
        };
        let timed_step_index_to_iter = |index: Index<TimedStep>| {
            once(index)
                .map(PatternDropRef::TimedStep)
                .map(Ok)
        };

        let pattern_drop_refs = |pattern: Pattern| {
            pattern_refs(
                pattern,
                arenas,
                PatternDropRef::PatternChain,
                PatternDropRef::TimedStepChain,
            )
        };

        match reference {
            PatternDropRef::Pattern(index) => {
                let pattern = arenas.get_pattern_arena().take(index)?;
                Ok(combine_iters(Some(pattern_drop_refs(pattern)), None))
            }
            PatternDropRef::TimedStep(index) => {
                let TimedStep(_, pattern_index) = arenas
                    .get_timed_step_arena()
                    .take(index)?;
                let iter = pattern_index_to_iter(pattern_index);
                Ok(combine_iters(None, Some(combine_iters(Some(iter), None))))
            }
            PatternDropRef::PatternChain(index) => {
                let pattern_index = arenas
                    .get_pattern_chain_arena()
                    .take(index)?
                    .0;
                let iter = pattern_index_to_iter(pattern_index);
                Ok(combine_iters(None, Some(combine_iters(Some(iter), None))))
            }
            PatternDropRef::TimedStepChain(index) => {
                let timed_step_index = arenas
                    .get_timed_step_chain_arena()
                    .take(index)?
                    .0;
                let iter = timed_step_index_to_iter(timed_step_index);
                Ok(combine_iters(None, Some(combine_iters(None, Some(iter)))))
            }
        }
    }
}
