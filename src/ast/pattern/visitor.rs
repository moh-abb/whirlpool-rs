use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;

pub trait PatternVisitor {
    type Output;
    type PatternOutput;

    fn get_arenas(&self) -> &impl PatternArenas;

    fn map_pattern(
        &self,
        pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output;
    fn map_cat(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput;
    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput;
    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput;
    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput;
    fn map_arrange(&self, multiple: Multiple<TimedStep>)
    -> Self::PatternOutput;

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput;
    fn map_silence(&self) -> Self::PatternOutput;
}

fn get_cloned_pattern(
    visitor: &impl PatternVisitor,
    pattern_index: Index<Pattern>,
) -> ArenaResult<Pattern> {
    visitor
        .get_arenas()
        .get_pattern_arena()
        .map(pattern_index.clone(), Clone::clone)
}

#[inline]
pub fn visit_pattern<V: PatternVisitor>(
    visitor: &V,
    pattern_index: Index<Pattern>,
) -> ArenaResult<V::Output> {
    let cloned_pattern = get_cloned_pattern(visitor, pattern_index.clone())?;
    let result = match cloned_pattern.clone() {
        Pattern::Cat(multiple)
        | Pattern::Seq(multiple)
        | Pattern::Stack(multiple) => {
            let map_func = match &cloned_pattern {
                Pattern::Cat(_) => V::map_cat,
                Pattern::Seq(_) => V::map_seq,
                Pattern::Stack(_) => V::map_stack,
                _ => unreachable!(),
            };
            map_func(visitor, multiple)
        }
        Pattern::TimeCat(multiple) | Pattern::Arrange(multiple) => {
            let map_func = match &cloned_pattern {
                Pattern::TimeCat(_) => V::map_time_cat,
                Pattern::Arrange(_) => V::map_arrange,
                _ => unreachable!(),
            };
            map_func(visitor, multiple)
        }
        Pattern::Note(note_unit) => visitor.map_note_unit(note_unit),
        Pattern::Silence => visitor.map_silence(),
    };
    Ok(visitor.map_pattern(pattern_index, result))
}

pub fn timed_step_iter(
    arenas: &impl PatternArenas,
    multiple: &Multiple<TimedStep>,
) -> impl DoubleEndedIterator<Item = ArenaResult<TimedStep>> {
    multiple
        .checked_iter(arenas.get_timed_step_chain_arena())
        .map(move |timed_step| {
            arenas
                .get_timed_step_arena()
                .map(timed_step?, Clone::clone)
        })
}
