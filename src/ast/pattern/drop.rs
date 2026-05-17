use core::mem;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::error::ArenaResult;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::structures::chain::Chain;
use crate::structures::index::INVALID_INDEX_VALUE;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;

#[derive(Debug)]
pub struct PatternDropAdapter<'a, Arenas: PatternArenas>(
    Option<Index<Pattern>>,
    &'a Arenas,
);

pub struct MultiplePatternDropAdapter<'a, Arenas: PatternArenas>(
    Option<Multiple<Pattern>>,
    &'a Arenas,
);

pub struct TimedStepDropAdapter<'a, Arenas: PatternArenas>(
    Option<Index<TimedStep>>,
    &'a Arenas,
);

pub struct MultipleTimedStepDropAdapter<'a, Arenas: PatternArenas>(
    Option<Multiple<TimedStep>>,
    &'a Arenas,
);

pub trait DropAdapter<'a, Item, Arenas> {
    fn new(item: Item, arenas: &'a Arenas) -> Self;

    fn take_item(&mut self) -> Item;
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Index<Pattern>, Arenas>
    for PatternDropAdapter<'a, Arenas>
{
    fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self(Some(index), arenas)
    }

    fn take_item(&mut self) -> Index<Pattern> {
        self.0.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Multiple<Pattern>, Arenas>
    for MultiplePatternDropAdapter<'a, Arenas>
{
    fn new(multiple: Multiple<Pattern>, arenas: &'a Arenas) -> Self {
        Self(Some(multiple), arenas)
    }

    fn take_item(&mut self) -> Multiple<Pattern> {
        self.0.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Index<TimedStep>, Arenas>
    for TimedStepDropAdapter<'a, Arenas>
{
    fn new(index: Index<TimedStep>, arenas: &'a Arenas) -> Self {
        Self(Some(index), arenas)
    }

    fn take_item(&mut self) -> Index<TimedStep> {
        self.0.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Multiple<TimedStep>, Arenas>
    for MultipleTimedStepDropAdapter<'a, Arenas>
{
    fn new(multiple: Multiple<TimedStep>, arenas: &'a Arenas) -> Self {
        Self(Some(multiple), arenas)
    }

    fn take_item(&mut self) -> Multiple<TimedStep> {
        self.0.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> Drop for PatternDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.0.take() {
            drop_pattern(index, self.1)
        }
    }
}

impl<'a, Arenas: PatternArenas> Drop
    for MultiplePatternDropAdapter<'a, Arenas>
{
    fn drop(&mut self) {
        if let Some(multiple) = self.0.take() {
            multiple_drop(
                multiple,
                self.1.get_pattern_chain_arena(),
                self.1,
                drop_pattern,
            );
        }
    }
}

pub fn multiple_cons<
    'a,
    Item: ArenaItem,
    Arenas: PatternArenas,
    ItemAdapter: DropAdapter<'a, Index<Item>, Arenas>,
    MultipleAdapter: DropAdapter<'a, Multiple<Item>, Arenas>,
>(
    mut multiple_adapter: MultipleAdapter,
    mut item_adapter: ItemAdapter,
    arenas: &'a Arenas,
    chain_arena: &impl Arena<Chain<Item>>,
) -> ArenaResult<MultipleAdapter> {
    // Allocation starts here.
    let invalid_chain = Chain(Index::new(INVALID_INDEX_VALUE), None, None);
    let alloc_chain = chain_arena.alloc(invalid_chain)?;
    // Allocation ends here.
    let mut multiple = multiple_adapter.take_item();
    chain_arena
        .inspect_mut(alloc_chain.clone(), |chain| {
            chain.0 = item_adapter.take_item();
        })
        .unwrap();
    multiple.push_back(chain_arena, alloc_chain);
    Ok(MultipleAdapter::new(multiple, arenas))
}

fn drop_pattern(pattern_index: Index<Pattern>, arenas: &impl PatternArenas) {
    visit_pattern(&PatternDropVisitor { arenas }, pattern_index)
}

fn drop_timed_step(
    timed_step_index: Index<TimedStep>,
    arenas: &impl PatternArenas,
) {
    let timed_step = arenas
        .get_timed_step_arena()
        .take(timed_step_index)
        .unwrap();
    let TimedStep(_time_unit, pattern_index) = timed_step;
    drop_pattern(pattern_index, arenas);
}

impl<'a, Arenas: PatternArenas> Drop for TimedStepDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.0.take() {
            drop_timed_step(index, self.1);
        }
    }
}

impl<'a, Arenas: PatternArenas> Drop
    for MultipleTimedStepDropAdapter<'a, Arenas>
{
    fn drop(&mut self) {
        if let Some(index) = self.0.take() {
            multiple_drop(
                index,
                self.1.get_timed_step_chain_arena(),
                self.1,
                drop_timed_step,
            );
        }
    }
}

/// Helper function to drop a [Multiple].
fn multiple_drop<Item: ArenaItem, Arenas: PatternArenas>(
    mut multiple: Multiple<Item>,
    chain_arena: &impl Arena<Chain<Item>>,
    arenas: &Arenas,
    drop_item: impl Fn(Index<Item>, &Arenas),
) {
    if multiple.is_empty() {
        return;
    }
    while let Some(end_index) = multiple.pop_back(chain_arena) {
        let taken_end = chain_arena.take(end_index).unwrap();
        let Chain(index, _end_prev, _end_next) = taken_end;
        debug_assert!(_end_prev.is_none());
        debug_assert!(_end_next.is_none());
        drop_item(index, arenas)
    }
}

struct PatternDropVisitor<'a, Arenas: PatternArenas> {
    arenas: &'a Arenas,
}

impl<'a, Arenas: PatternArenas> PatternVisitor
    for PatternDropVisitor<'a, Arenas>
{
    type Output = ();
    type PatternOutput = ();

    fn get_arenas(&self) -> &impl PatternArenas {
        self.arenas
    }

    fn map_pattern(
        &self,
        pattern_index: Index<Pattern>,
        _: Self::PatternOutput,
    ) -> Self::Output {
        self.arenas
            .get_pattern_arena()
            .take(pattern_index)
            .unwrap();
    }

    fn map_cat(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        mem::drop(MultiplePatternDropAdapter::new(multiple, self.arenas));
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        mem::drop(MultiplePatternDropAdapter::new(multiple, self.arenas));
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        mem::drop(MultiplePatternDropAdapter::new(multiple, self.arenas));
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        mem::drop(MultipleTimedStepDropAdapter::new(multiple, self.arenas));
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        mem::drop(MultipleTimedStepDropAdapter::new(multiple, self.arenas));
    }

    fn map_note_unit(&self, _: NoteUnit) -> Self::PatternOutput {}

    fn map_silence(&self) -> Self::PatternOutput {}
}
