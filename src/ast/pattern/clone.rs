use core::iter::once;

use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::refs::PatternCloneRef;
use crate::ast::pattern::refs::PatternDropRef;
use crate::ast::pattern::refs::combine_iters;
use crate::ast::pattern::refs::pattern_refs;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::INVALID_INDEX_VALUE;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::clone::clone_in_arenas;
use crate::mem::clone::refs::CloneRefs;

#[derive(Debug)]
pub struct PatternCloneDropAdapter<'a, Arenas: PatternArenas>(
    ArenaResult<Option<Index<Pattern>>>,
    &'a Arenas,
);

impl<'a, Arenas: PatternArenas> PatternCloneDropAdapter<'a, Arenas> {
    pub fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self(Ok(Some(index)), arenas)
    }

    pub fn take_opt_item(&mut self) -> Option<Index<Pattern>> {
        self.0
            .as_mut()
            .ok()
            .map(Option::take)
            .flatten()
    }

    pub fn take_item(&mut self) -> Index<Pattern> {
        self.take_opt_item().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> Drop for PatternCloneDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.take_opt_item() {
            drop(PatternDropAdapter(Some(index), self.1));
        }
    }
}

impl<'a, Arenas: PatternArenas> Clone for PatternCloneDropAdapter<'a, Arenas> {
    fn clone(&self) -> Self {
        let cloned_index = match self.0.clone() {
            Ok(Some(index)) => {
                let clone_result = clone_in_arenas(index, self.1);
                clone_result.map(|drop_ref| {
                    let PatternDropRef::Pattern(index) = drop_ref else {
                        unreachable!()
                    };
                    Some(index)
                })
            }
            other => other,
        };

        Self(cloned_index, self.1)
    }
}

fn clone_pattern_with_empty_multiple(
    index: Index<Pattern>,
    arenas: &impl PatternArenas,
) -> ArenaResult<Index<Pattern>> {
    let pattern_arena = arenas.get_pattern_arena();
    let cloned_pattern = pattern_arena.inspect(index, Clone::clone)?;
    let cloned_empty_pattern = match cloned_pattern {
        Pattern::Cat(_) => Pattern::Cat(Multiple::new_empty()),
        Pattern::Seq(_) => Pattern::Seq(Multiple::new_empty()),
        Pattern::Stack(_) => Pattern::Stack(Multiple::new_empty()),
        Pattern::TimeCat(_) => Pattern::TimeCat(Multiple::new_empty()),
        Pattern::Arrange(_) => Pattern::Arrange(Multiple::new_empty()),
        Pattern::Note(note_unit) => Pattern::Note(note_unit),
        Pattern::Silence => Pattern::Silence,
    };
    pattern_arena.push(cloned_empty_pattern)
}

impl<Arenas: PatternArenas> CloneRefs<Arenas> for Index<Pattern> {
    type CloneRefType = PatternCloneRef;

    fn start_and_result_ref<'a>(
        start: Self,
        arenas: &'a Arenas,
    ) -> ArenaResult<(Self::CloneRefType, Self::DropRefType)> {
        let cloned_start =
            clone_pattern_with_empty_multiple(start.clone(), arenas)?;
        Ok((
            PatternCloneRef::PatternRoot {
                orig_index: start,
                root_index: cloned_start.clone(),
            },
            PatternDropRef::Pattern(cloned_start),
        ))
    }

    fn process_ref<'a>(
        reference: Self::CloneRefType,
        arenas: &'a Arenas,
    ) -> ArenaResult<impl Iterator<Item = ArenaResult<Self::CloneRefType>> + 'a>
    {
        // Example structures:
        //
        // Cat(A, B, C):
        //
        // I0 (PatternRoot)
        // -> Cat(C1, C2, C3)
        //    -> C1 (PatternChain)
        //       -> I1 (PatternChildOfPattern: A)
        //    -> C2 (PatternChain)
        //       -> I2 (PatternChildOfPattern: B)
        //    -> C3 (PatternChain)
        //       -> I3 (PatternChildOfPattern: C)
        //
        // TimeCat([0.5, A], [0.25, B], [0.25, C]):
        // I0 (PatternRoot)
        // -> TimeCat(C1, C2, C3)
        //    -> C1 (TimedStepChain)
        //       -> I1 (TimedStep: [0.5, A])
        //          -> I2 (PatternChildOfTimedStep: A)
        //    -> ... (repeated for [0.25, B] and [0.25, C])

        let pattern_arena = arenas.get_pattern_arena();
        let timed_step_arena = arenas.get_timed_step_arena();

        let pattern_clone_refs =
            |orig_pattern: Pattern, parent: Index<Pattern>| {
                let cloned_parent = parent.clone();
                let cloned_parent_2 = parent.clone();
                pattern_refs(
                    orig_pattern,
                    arenas,
                    move |index| PatternCloneRef::PatternChain {
                        index,
                        parent: cloned_parent.clone(),
                    },
                    move |index| PatternCloneRef::TimedStepChain {
                        index,
                        parent: cloned_parent_2.clone(),
                    },
                )
            };

        let append_pattern_chain =
            |parent: Index<Pattern>, chain_index: Index<Chain<Pattern>>| {
                pattern_arena.inspect_mut(parent, |pattern| match pattern {
                    Pattern::Cat(multiple)
                    | Pattern::Seq(multiple)
                    | Pattern::Stack(multiple) => multiple.push_front(
                        arenas.get_pattern_chain_arena(),
                        chain_index,
                    ),
                    _ => unreachable!(),
                })
            };

        let append_timed_step_chain =
            |parent: Index<Pattern>, chain_index: Index<Chain<TimedStep>>| {
                pattern_arena.inspect_mut(parent, |pattern| match pattern {
                    Pattern::TimeCat(multiple) | Pattern::Arrange(multiple) => {
                        multiple.push_front(
                            arenas.get_timed_step_chain_arena(),
                            chain_index,
                        )
                    }
                    _ => unreachable!(),
                })
            };

        match reference {
            PatternCloneRef::PatternRoot { orig_index, root_index } => {
                let cloned_orig_pattern =
                    pattern_arena.inspect(orig_index.clone(), Clone::clone)?;
                let iter =
                    Some(pattern_clone_refs(cloned_orig_pattern, root_index));
                Ok(combine_iters(iter, None))
            }
            PatternCloneRef::PatternChildOfPattern { index, parent } => {
                let cloned_pattern =
                    pattern_arena.inspect(index.clone(), Clone::clone)?;
                let alloc_pattern =
                    clone_pattern_with_empty_multiple(index, arenas)?;
                // Set the chain to point to the allocated pattern.
                arenas
                    .get_pattern_chain_arena()
                    .inspect_mut(parent, |Chain(index, _, _)| {
                        debug_assert_eq!(
                            *index,
                            Index::new(INVALID_INDEX_VALUE)
                        );
                        *index = alloc_pattern.clone();
                    })?;
                let iter =
                    Some(pattern_clone_refs(cloned_pattern, alloc_pattern));
                Ok(combine_iters(iter, None))
            }
            PatternCloneRef::PatternChildOfTimedStep { index, parent } => {
                let cloned_pattern =
                    pattern_arena.inspect(index.clone(), Clone::clone)?;
                let alloc_pattern =
                    clone_pattern_with_empty_multiple(index, arenas)?;
                // Set the timed step to point to the allocated pattern.
                arenas
                    .get_timed_step_arena()
                    .inspect_mut(parent, |TimedStep(_duration, index)| {
                        debug_assert_eq!(
                            *index,
                            Index::new(INVALID_INDEX_VALUE)
                        );
                        *index = alloc_pattern.clone();
                    })?;
                let iter =
                    Some(pattern_clone_refs(cloned_pattern, alloc_pattern));
                Ok(combine_iters(iter, None))
            }
            PatternCloneRef::TimedStep { index, parent } => {
                let TimedStep(duration, child_pattern) =
                    timed_step_arena.inspect(index.clone(), Clone::clone)?;
                let cloned_timed_step =
                    TimedStep(duration, Index::new(INVALID_INDEX_VALUE));
                let alloc_timed_step =
                    timed_step_arena.push(cloned_timed_step.clone())?;
                // Set the chain to point to the allocated pattern.
                arenas
                    .get_timed_step_chain_arena()
                    .inspect_mut(parent, |Chain(index, _, _)| {
                        debug_assert_eq!(
                            *index,
                            Index::new(INVALID_INDEX_VALUE)
                        );
                        *index = alloc_timed_step.clone();
                    })?;
                let iter = once(Ok(PatternCloneRef::PatternChildOfTimedStep {
                    index: child_pattern,
                    parent: alloc_timed_step,
                }));
                let combined_iter = Some(combine_iters(Some(iter), None));
                Ok(combine_iters(None, combined_iter))
            }
            PatternCloneRef::PatternChain { index, parent } => {
                let chain_arena = arenas.get_pattern_chain_arena();
                let child = chain_arena
                    .inspect(index, |Chain(child, _, _)| child.clone())?;
                // Allocate a new chain with an invalid index.
                let invalid_chain =
                    Chain(Index::new(INVALID_INDEX_VALUE), None, None);
                let chain_index = chain_arena.push(invalid_chain)?;
                // Append the pattern chain to the parent.
                append_pattern_chain(parent, chain_index.clone())?;
                let iter = once(Ok(PatternCloneRef::PatternChildOfPattern {
                    index: child,
                    parent: chain_index,
                }));
                let combined_iter = Some(combine_iters(Some(iter), None));
                Ok(combine_iters(None, combined_iter))
            }
            PatternCloneRef::TimedStepChain { index, parent } => {
                let chain_arena = arenas.get_timed_step_chain_arena();
                let child = chain_arena
                    .inspect(index, |Chain(child, _, _)| child.clone())?;
                // Allocate a new chain with an invalid index.
                let invalid_chain =
                    Chain(Index::new(INVALID_INDEX_VALUE), None, None);
                let chain_index = chain_arena.push(invalid_chain)?;
                // Append the pattern chain to the parent.
                append_timed_step_chain(parent, chain_index.clone())?;
                let iter = once(Ok(PatternCloneRef::TimedStep {
                    index: child,
                    parent: chain_index,
                }));
                let combined_iter = Some(combine_iters(None, Some(iter)));
                Ok(combine_iters(None, combined_iter))
            }
        }
    }
}
