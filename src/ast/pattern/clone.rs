use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::error::ArenaResult;
use crate::arena::index::INVALID_INDEX_VALUE;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::drop::DropAdapter;
use crate::ast::pattern::drop::PatternChainDropAdapter;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::drop::TimedStepChainDropAdapter;
use crate::ast::pattern::drop::TimedStepDropAdapter;
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

    pub fn clone_pattern(
        &self,
        pattern_index: Index<Pattern>,
    ) -> ArenaResult<Index<Pattern>> {
        Ok(visit_pattern(self, pattern_index)?.take_index())
    }
}

#[derive(Debug)]
pub struct PatternCloneDropAdapter<'a, Arenas: PatternArenas>(
    ArenaResult<CloneAdapterInner<'a, Arenas>>,
);

#[derive(Debug)]
pub struct CloneAdapterInner<'a, Arenas: PatternArenas> {
    index: Option<Index<Pattern>>,
    arenas: &'a Arenas,
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Pattern, Arenas>
    for PatternCloneDropAdapter<'a, Arenas>
{
    fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self(Ok(CloneAdapterInner { index: Some(index), arenas }))
    }

    fn take_index(&mut self) -> Index<Pattern> {
        self.0
            .as_mut()
            .unwrap()
            .index
            .take()
            .unwrap()
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
            let cloned_pattern =
                clone_visitor.clone_pattern(inner.index.clone().unwrap())?;
            Ok(CloneAdapterInner {
                index: Some(cloned_pattern),
                arenas: inner.arenas,
            })
        }))
    }
}

fn alloc_pattern<'a, ChainItem: ArenaItem, Arenas: PatternArenas>(
    arenas: &'a Arenas,
    length: u16,
    make_invalid: impl FnOnce(Multiple<ChainItem>) -> Pattern,
    pattern_to_multiple: impl FnOnce(&mut Pattern) -> &mut Multiple<ChainItem>,
    output: ArenaResult<impl DropAdapter<'a, Chain<ChainItem>, Arenas>>,
) -> ArenaResult<PatternDropAdapter<'a, Arenas>> {
    let pattern_arena = arenas.get_pattern_arena();
    let invalid_pattern = make_invalid(Multiple {
        length,
        index: Index::new(INVALID_INDEX_VALUE),
    });
    let cloned_pattern_index = pattern_arena.alloc(invalid_pattern)?;
    let mut cloned_chain_index = output?;
    let insert_chain_index = |pattern: &mut Pattern| {
        let multiple = pattern_to_multiple(pattern);
        let new_multiple =
            Multiple { length, index: cloned_chain_index.take_index() };
        let _ = core::mem::replace(multiple, new_multiple);
    };
    pattern_arena
        .inspect_mut(cloned_pattern_index.clone(), insert_chain_index)
        .unwrap();
    Ok(DropAdapter::new(cloned_pattern_index, arenas))
}

pub fn make_chain_output_nil<
    'a,
    Item: ArenaItem,
    Arenas: PatternArenas,
    ChainDropAdapter: DropAdapter<'a, Chain<Item>, Arenas>,
>(
    arenas: &'a Arenas,
    chain_arena: &'a impl Arena<Chain<Item>>,
) -> ArenaResult<ChainDropAdapter> {
    let chain_index = chain_arena.alloc(Chain::Nil)?;
    Ok(ChainDropAdapter::new(chain_index, arenas))
}

pub fn make_chain_output_cons<
    'a,
    Item: ArenaItem,
    Arenas: PatternArenas,
    HeadDropAdapter: DropAdapter<'a, Item, Arenas>,
    ChainDropAdapter: DropAdapter<'a, Chain<Item>, Arenas>,
>(
    arenas: &'a Arenas,
    chain_arena: &'a impl Arena<Chain<Item>>,
    chain_output: ArenaResult<ChainDropAdapter>,
    make_head: impl FnOnce() -> ArenaResult<HeadDropAdapter>,
) -> ArenaResult<ChainDropAdapter> {
    // If allocation of the tail patterns have failed, don't attempt to make
    // a `Cons`; hence the `?`.
    let mut tail_adapter = chain_output?;
    let mut cloned_head_adapter = make_head()?;
    let cons = Chain::Cons {
        head: Index::new(INVALID_INDEX_VALUE),
        tail: Index::new(INVALID_INDEX_VALUE),
    };
    let chain_index = chain_arena.alloc(cons)?;
    let insert_cons_indices = |cons: &mut Chain<_>| {
        let Chain::Cons { head, tail } = cons else { unreachable!() };
        let inserted_head = cloned_head_adapter.take_index();
        let inserted_tail = tail_adapter.take_index();
        let _ = core::mem::replace(head, inserted_head);
        let _ = core::mem::replace(tail, inserted_tail);
    };
    chain_arena
        .inspect_mut(chain_index.clone(), insert_cons_indices)
        .unwrap();
    Ok(ChainDropAdapter::new(chain_index, arenas))
}

impl<'a, Arenas: PatternArenas> PatternVisitor for CloneVisitor<'a, Arenas> {
    type Output = ArenaResult<PatternDropAdapter<'a, Arenas>>;
    type PatternOutput = ArenaResult<PatternDropAdapter<'a, Arenas>>;
    type PatternChainOutput = ArenaResult<PatternChainDropAdapter<'a, Arenas>>;
    type TimedStepOutput = ArenaResult<Index<TimedStep>>;
    type TimedStepChainOutput =
        ArenaResult<TimedStepChainDropAdapter<'a, Arenas>>;

    const CHAINS_FOLD_RIGHT: bool = true;

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

    fn map_cat(
        &self,
        multiple: Multiple<Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        alloc_pattern(
            self.arenas,
            multiple.length,
            Pattern::Cat,
            |pattern| {
                let Pattern::Cat(index) = pattern else { unreachable!() };
                index
            },
            pattern_chain_output,
        )
    }

    fn map_seq(
        &self,
        multiple: Multiple<Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        alloc_pattern(
            self.arenas,
            multiple.length,
            Pattern::Seq,
            |pattern| {
                let Pattern::Seq(index) = pattern else { unreachable!() };
                index
            },
            pattern_chain_output,
        )
    }

    fn map_stack(
        &self,
        multiple: Multiple<Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        alloc_pattern(
            self.arenas,
            multiple.length,
            Pattern::Stack,
            |pattern| {
                let Pattern::Stack(index) = pattern else { unreachable!() };
                index
            },
            pattern_chain_output,
        )
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput {
        alloc_pattern(
            self.arenas,
            multiple.length,
            Pattern::TimeCat,
            |pattern| {
                let Pattern::TimeCat(index) = pattern else { unreachable!() };
                index
            },
            timed_step_chain_output,
        )
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

    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput {
        make_chain_output_nil(
            self.arenas,
            self.arenas.get_pattern_chain_arena(),
        )
    }

    fn fold_pattern_chain_output(
        &self,
        pattern_chain_output: Self::PatternChainOutput,
        _next_chain_index: Index<Chain<Pattern>>,
        pattern_index: Index<Pattern>,
    ) -> Self::PatternChainOutput {
        let clone_pattern =
            || visit_pattern(&Self { arenas: self.arenas }, pattern_index);
        make_chain_output_cons(
            self.arenas,
            self.arenas.get_pattern_chain_arena(),
            pattern_chain_output,
            clone_pattern,
        )
    }

    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput {
        make_chain_output_nil(
            self.arenas,
            self.arenas.get_timed_step_chain_arena(),
        )
    }

    fn fold_timed_step_chain_output(
        &self,
        timed_step_chain_output: Self::TimedStepChainOutput,
        _next_chain_index: Index<Chain<TimedStep>>,
        timed_step_index: Index<TimedStep>,
    ) -> Self::TimedStepChainOutput {
        let clone_timed_step = || {
            let timed_step_arena = self.arenas.get_timed_step_arena();
            let cloned_timed_step = timed_step_arena
                .inspect(timed_step_index, Clone::clone)
                .unwrap();
            Ok(TimedStepDropAdapter::new(
                timed_step_arena.alloc(cloned_timed_step)?,
                self.arenas,
            ))
        };
        make_chain_output_cons(
            self.arenas,
            self.arenas.get_timed_step_chain_arena(),
            timed_step_chain_output,
            clone_timed_step,
        )
    }
}
