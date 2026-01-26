use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::error::ArenaResult;
use crate::arena::index::INVALID_INDEX_VALUE;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::MultiplePatternDropAdapter;
use crate::ast::pattern::drop::MultipleTimedStepDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::drop::TimedStepDropAdapter;
use crate::ast::pattern::drop::multiple_cons;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;

pub struct CloneVisitor<'a, Arenas> {
    arenas: &'a Arenas,
}

impl<'a, Arenas: PatternArenas> CloneVisitor<'a, Arenas> {
    pub fn new(arenas: &'a Arenas) -> Self {
        Self { arenas }
    }

    fn clone_multiple_pattern(
        &self,
        orig_multiple: Multiple<Pattern>,
        make_pattern: impl FnOnce(Multiple<Pattern>) -> Pattern,
        get_multiple: impl FnOnce(&mut Pattern) -> &mut Multiple<Pattern>,
    ) -> ArenaResult<PatternDropAdapter<'a, Arenas>> {
        let chain_arena = self.arenas.get_pattern_chain_arena();
        let cloned_multiple = orig_multiple.fold_left(
            chain_arena,
            empty_multiple(self.arenas),
            |acc: ArenaResult<MultiplePatternDropAdapter<'a, _>>, x| {
                let pattern_adapter = visit_pattern(self, x)?;
                multiple_cons(acc?, pattern_adapter, self.arenas, chain_arena)
            },
        );
        alloc_pattern(self.arenas, make_pattern, get_multiple, cloned_multiple?)
    }

    fn clone_multiple_timed_step(
        &self,
        orig_multiple: Multiple<TimedStep>,
        make_pattern: impl FnOnce(Multiple<TimedStep>) -> Pattern,
        get_multiple: impl FnOnce(&mut Pattern) -> &mut Multiple<TimedStep>,
    ) -> ArenaResult<PatternDropAdapter<'a, Arenas>> {
        let timed_step_arena = self.arenas.get_timed_step_arena();
        let chain_arena = self.arenas.get_timed_step_chain_arena();
        let clone_timed_step = |timed_step: Index<TimedStep>| {
            let cloned_timed_step = timed_step_arena
                .inspect(timed_step, Clone::clone)
                .unwrap();
            let TimedStep(time_unit, pattern_index) = cloned_timed_step;
            let mut cloned_pattern = visit_pattern(self, pattern_index)?;
            // Allocation starts here.
            let invalid_timed_step =
                TimedStep(time_unit, Index::new(INVALID_INDEX_VALUE));
            let alloc_timed_step = timed_step_arena
                .alloc(invalid_timed_step)
                .unwrap();
            // Allocation ends here.
            timed_step_arena
                .inspect_mut(alloc_timed_step.clone(), |timed_step| {
                    timed_step.1 = cloned_pattern.take_item();
                })
                .unwrap();
            ArenaResult::Ok(TimedStepDropAdapter::new(
                alloc_timed_step,
                self.arenas,
            ))
        };
        let cloned_multiple = orig_multiple.fold_left(
            chain_arena,
            empty_multiple(self.arenas),
            |acc: ArenaResult<MultipleTimedStepDropAdapter<'a, _>>, x| {
                let cloned_x = clone_timed_step(x)?;
                multiple_cons(acc?, cloned_x, self.arenas, chain_arena)
            },
        );
        alloc_pattern(self.arenas, make_pattern, get_multiple, cloned_multiple?)
    }
}

#[derive(Debug)]
pub struct PatternCloneDropAdapter<'a, Arenas: PatternArenas>(
    ArenaResult<CloneAdapterInner<'a, Arenas>>,
);

impl<'a, Arenas: PatternArenas> PatternCloneDropAdapter<'a, Arenas> {
    fn try_take_index(&mut self) -> ArenaResult<Index<Pattern>> {
        let inner = self.0.as_mut().map_err(|e| *e);
        Ok(inner?.index.take().unwrap())
    }
}

#[derive(Debug)]
struct CloneAdapterInner<'a, Arenas: PatternArenas> {
    index: Option<Index<Pattern>>,
    arenas: &'a Arenas,
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Index<Pattern>, Arenas>
    for PatternCloneDropAdapter<'a, Arenas>
{
    fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self(Ok(CloneAdapterInner { index: Some(index), arenas }))
    }

    fn take_item(&mut self) -> Index<Pattern> {
        self.try_take_index().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> Drop for PatternCloneDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Ok(inner) = &mut self.0
            && let Some(index) = &inner.index
        {
            core::mem::drop(PatternDropAdapter::new(
                index.clone(),
                inner.arenas,
            ))
        }
    }
}

impl<'a, Arenas: PatternArenas> Clone for PatternCloneDropAdapter<'a, Arenas> {
    fn clone(&self) -> Self {
        let inner_res = self.0.as_ref().map_err(Clone::clone);
        Self(inner_res.and_then(|inner| {
            let clone_visitor = CloneVisitor::new(inner.arenas);
            let mut cloned_pattern =
                visit_pattern(&clone_visitor, inner.index.clone().unwrap())?;
            let index = cloned_pattern.take_item();
            Ok(CloneAdapterInner { index: Some(index), arenas: inner.arenas })
        }))
    }
}

pub fn alloc_pattern<'a, Item: ArenaItem, Arenas: PatternArenas>(
    arenas: &'a Arenas,
    make_invalid_pattern: impl FnOnce(Multiple<Item>) -> Pattern,
    pattern_to_multiple: impl FnOnce(&mut Pattern) -> &mut Multiple<Item>,
    mut multiple_adapter: impl DropAdapter<'a, Multiple<Item>, Arenas>,
) -> ArenaResult<PatternDropAdapter<'a, Arenas>> {
    let pattern_arena = arenas.get_pattern_arena();
    let invalid_pattern = make_invalid_pattern(Multiple::new_empty());
    // Allocation starts here.
    let alloc_pattern_index = pattern_arena.alloc(invalid_pattern)?;
    // Allocation ends here.
    let insert_multiple = |pattern: &mut Pattern| {
        let multiple = pattern_to_multiple(pattern);
        let new_multiple = multiple_adapter.take_item();
        let _ = core::mem::replace(multiple, new_multiple);
    };
    pattern_arena
        .inspect_mut(alloc_pattern_index.clone(), insert_multiple)
        .unwrap();
    Ok(DropAdapter::new(alloc_pattern_index, arenas))
}

fn empty_multiple<
    'a,
    Item: ArenaItem,
    Arenas: PatternArenas,
    MultipleDropAdapter: DropAdapter<'a, Multiple<Item>, Arenas>,
>(
    arenas: &'a Arenas,
) -> ArenaResult<MultipleDropAdapter> {
    Ok(MultipleDropAdapter::new(Multiple::new_empty(), arenas))
}

impl<'a, Arenas: PatternArenas> PatternVisitor for CloneVisitor<'a, Arenas> {
    type Output = ArenaResult<PatternDropAdapter<'a, Arenas>>;
    type PatternOutput = ArenaResult<PatternDropAdapter<'a, Arenas>>;

    fn get_arenas(&self) -> &impl PatternArenas {
        self.arenas
    }

    fn map_pattern(
        &self,
        _: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output {
        pattern_output
    }

    fn map_cat(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        self.clone_multiple_pattern(multiple, Pattern::Cat, |pattern| {
            let Pattern::Cat(multiple) = pattern else { unreachable!() };
            multiple
        })
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        self.clone_multiple_pattern(multiple, Pattern::Seq, |pattern| {
            let Pattern::Seq(multiple) = pattern else { unreachable!() };
            multiple
        })
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        self.clone_multiple_pattern(multiple, Pattern::Stack, |pattern| {
            let Pattern::Stack(multiple) = pattern else { unreachable!() };
            multiple
        })
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        self.clone_multiple_timed_step(multiple, Pattern::TimeCat, |pattern| {
            let Pattern::TimeCat(multiple) = pattern else { unreachable!() };
            multiple
        })
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let pattern_index = self
            .arenas
            .get_pattern_arena()
            .alloc(Pattern::Note(unit))?;
        Ok(PatternDropAdapter::new(pattern_index, self.arenas))
    }

    fn map_silence(&self) -> Self::PatternOutput {
        let pattern_index = self
            .arenas
            .get_pattern_arena()
            .alloc(Pattern::Silence)?;
        Ok(PatternDropAdapter::new(pattern_index, self.arenas))
    }
}
