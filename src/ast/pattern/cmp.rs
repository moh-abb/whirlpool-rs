use core::cmp::Ordering;
use core::iter::once;
use core::ops::ControlFlow;

use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::pattern_discriminant;
use crate::ast::pattern::refs::PatternOrdRef;
use crate::ast::pattern::refs::combine_iters;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::cmp::cmp_in_arenas;
use crate::mem::cmp::refs::NextOrdRefs;
use crate::mem::cmp::refs::OrdRefs;

fn multiple_pattern_iter(
    multiple: Multiple<Pattern>,
    arenas: &impl PatternArenas,
) -> impl Iterator<Item = ArenaResult<PatternOrdRef>> {
    multiple
        .checked_iter(arenas.get_pattern_chain_arena())
        .map(|result| result.map(PatternOrdRef::Pattern))
}

fn multiple_timed_step_iter(
    multiple: Multiple<TimedStep>,
    arenas: &impl PatternArenas,
) -> impl Iterator<Item = ArenaResult<PatternOrdRef>> {
    multiple
        .checked_iter(arenas.get_timed_step_chain_arena())
        .map(|result| result.map(PatternOrdRef::TimedStep))
}

impl<ArenasX: PatternArenas, ArenasY: PatternArenas> OrdRefs<ArenasX, ArenasY>
    for Index<Pattern>
{
    type OrdRefType = PatternOrdRef;

    fn start_refs(
        start_x: Self,
        start_y: Self,
    ) -> (Self::OrdRefType, Self::OrdRefType) {
        (PatternOrdRef::Pattern(start_x), PatternOrdRef::Pattern(start_y))
    }

    fn process_ref<'a>(
        reference_x: Self::OrdRefType,
        reference_y: Self::OrdRefType,
        arenas_x: &'a ArenasX,
        arenas_y: &'a ArenasY,
    ) -> ArenaResult<
        ControlFlow<Ordering, impl NextOrdRefs<'a, ArenasX, ArenasY, Self>>,
    > {
        let break_value = |ordering: Ordering| Ok(ControlFlow::Break(ordering));

        let process_patterns = |[index_x, index_y]: [Index<Pattern>; 2]| {
            let cloned_x = arenas_x
                .get_pattern_arena()
                .inspect(index_x, Clone::clone)?;
            let cloned_y = arenas_y
                .get_pattern_arena()
                .inspect(index_y, Clone::clone)?;

            let [discr_x, discr_y] =
                [&cloned_x, &cloned_y].map(pattern_discriminant);
            if discr_x != discr_y {
                return break_value(discr_x.cmp(&discr_y));
            }
            let iter = match (cloned_x, cloned_y) {
                (Pattern::Cat(multiple_x), Pattern::Cat(multiple_y))
                | (Pattern::Seq(multiple_x), Pattern::Seq(multiple_y))
                | (Pattern::Stack(multiple_x), Pattern::Stack(multiple_y)) => {
                    let [len_x, len_y] =
                        [&multiple_x, &multiple_y].map(Multiple::length);
                    if len_x != len_y {
                        return break_value(len_x.cmp(&len_y));
                    }

                    let iter = multiple_pattern_iter(multiple_x, arenas_x)
                        .zip(multiple_pattern_iter(multiple_y, arenas_y));

                    combine_iters(Some(iter), None)
                }
                (
                    Pattern::TimeCat(multiple_x),
                    Pattern::TimeCat(multiple_y),
                )
                | (
                    Pattern::Arrange(multiple_x),
                    Pattern::Arrange(multiple_y),
                ) => {
                    let [len_x, len_y] =
                        [&multiple_x, &multiple_y].map(Multiple::length);
                    if len_x != len_y {
                        return break_value(discr_x.cmp(&discr_y));
                    }

                    let iter = multiple_timed_step_iter(multiple_x, arenas_x)
                        .zip(multiple_timed_step_iter(multiple_y, arenas_y));

                    combine_iters(None, Some(iter))
                }
                (Pattern::Note(note_unit_x), Pattern::Note(note_unit_y)) => {
                    return break_value(note_unit_x.cmp(&note_unit_y));
                }
                (Pattern::Silence, Pattern::Silence) => {
                    return break_value(Ordering::Equal);
                }
                _ => unreachable!(),
            };

            Ok(ControlFlow::Continue(combine_iters(Some(iter), None)))
        };

        match (reference_x, reference_y) {
            (
                PatternOrdRef::Pattern(index_x),
                PatternOrdRef::Pattern(index_y),
            ) => process_patterns([index_x, index_y]),
            (
                PatternOrdRef::TimedStep(index_x),
                PatternOrdRef::TimedStep(index_y),
            ) => {
                let TimedStep(x_dur, x_pat) = arenas_x
                    .get_timed_step_arena()
                    .inspect(index_x, Clone::clone)?;
                let TimedStep(y_dur, y_pat) = arenas_y
                    .get_timed_step_arena()
                    .inspect(index_y, Clone::clone)?;

                if x_dur != y_dur {
                    return break_value(x_dur.cmp(&y_dur));
                }

                let pattern_to_iter = |pattern_index| {
                    once(pattern_index)
                        .map(PatternOrdRef::Pattern)
                        .map(Ok)
                };
                let iter = pattern_to_iter(x_pat).zip(pattern_to_iter(y_pat));

                Ok(ControlFlow::Continue(combine_iters(None, Some(iter))))
            }
            _ => unreachable!(),
        }
    }
}

/// Used to compare two [Pattern]s, possibly from two different arenas.
#[derive(Debug)]
pub struct PatternOrdAdapter<'a, Arenas1, Arenas2> {
    index: Index<Pattern>,
    arenas: Result<&'a Arenas1, &'a Arenas2>,
}

impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas>
    PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    #[allow(unused)]
    pub fn new_left(index: Index<Pattern>, arenas: &'a Arenas1) -> Self {
        Self { index, arenas: Ok(arenas) }
    }

    #[allow(unused)]
    pub fn new_right(index: Index<Pattern>, arenas: &'a Arenas2) -> Self {
        Self { index, arenas: Err(arenas) }
    }
}

impl<'a, Arenas: PatternArenas> PatternOrdAdapter<'a, Arenas, Arenas> {
    #[allow(unused)]
    pub fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self { index, arenas: Ok(arenas) }
    }
}

/// Implementation of Ord for `Index<Pattern>`'s adapter
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> Ord
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn cmp(&self, other: &Self) -> Ordering {
        let i1 = self.index.clone();
        let i2 = other.index.clone();
        let ordering_result = match (self.arenas, other.arenas) {
            (Ok(a1), Ok(a2)) => cmp_in_arenas(i1, i2, a1, a2),
            (Ok(a1), Err(a2)) => cmp_in_arenas(i1, i2, a1, a2),
            (Err(a1), Ok(a2)) => cmp_in_arenas(i1, i2, a1, a2),
            (Err(a1), Err(a2)) => cmp_in_arenas(i1, i2, a1, a2),
        };
        ordering_result.unwrap()
    }
}

/// Default implementation of PartialEq for `Index<Pattern>`'s adapter
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PartialEq
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

/// Default implementation of [Eq] for `Index<Pattern>`'s adapter.
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> Eq
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
}

/// Default implementation of [PartialOrd] for `Index<Pattern>`'s adapter.
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PartialOrd
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
