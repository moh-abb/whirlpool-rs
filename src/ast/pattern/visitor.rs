use crate::arena::Arena;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;

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

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput;
    fn map_silence(&self) -> Self::PatternOutput;
}

fn get_cloned_pattern(
    visitor: &impl PatternVisitor,
    pattern_index: Index<Pattern>,
) -> Pattern {
    let panic_with_err = |err| {
        panic!(
            "[visit_pattern]: accessing pattern at {pattern_index:?} gave error {err:?}",
        )
    };
    visitor
        .get_arenas()
        .get_pattern_arena()
        .inspect(pattern_index.clone(), Clone::clone)
        .unwrap_or_else(panic_with_err)
}

#[inline]
pub fn visit_pattern<V: PatternVisitor>(
    visitor: &V,
    pattern_index: Index<Pattern>,
) -> V::Output {
    let cloned_pattern = get_cloned_pattern(visitor, pattern_index.clone());
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
        Pattern::TimeCat(multiple) => {
            let map_func = V::map_time_cat;
            map_func(visitor, multiple)
        }
        Pattern::Note(note_unit) => visitor.map_note_unit(note_unit),
        Pattern::Silence => visitor.map_silence(),
    };
    visitor.map_pattern(pattern_index, result)
}
