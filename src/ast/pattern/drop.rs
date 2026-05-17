use core::iter::once;

use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::INVALID_INDEX_VALUE;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::drop::drop_in_arenas;
use crate::mem::drop::refs::DropRefs;

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
            debug_unwrap(drop_in_arenas(DropRef::Pattern(index), self.1))
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
                DropRef::Pattern,
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

impl<'a, Arenas: PatternArenas> Drop for TimedStepDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.0.take() {
            debug_unwrap(drop_in_arenas(DropRef::TimedStep(index), self.1))
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
                DropRef::TimedStep,
            )
        }
    }
}

fn debug_unwrap(result: ArenaResult<()>) {
    if cfg!(debug_assertions) {
        result.unwrap()
    }
}

/// Helper function to drop a [Multiple].
fn multiple_drop<Item: ArenaItem, Arenas: PatternArenas>(
    multiple: Multiple<Item>,
    chain_arena: &impl Arena<Chain<Item>>,
    arenas: &Arenas,
    make_drop_ref: impl Fn(Index<Item>) -> DropRef,
) {
    let drop_result = multiple
        .iter(chain_arena)
        .try_for_each(|item| drop_in_arenas(make_drop_ref(item), arenas));
    debug_unwrap(drop_result)
}

/// Module to avoid leaking type `DropRef`.
mod private {
    use crate::ast::Pattern;
    use crate::ast::TimedStep;
    use crate::mem::Chain;
    use crate::mem::Index;

    /// The references that will recursively need to be dropped when a
    /// `Pattern` is dropped.
    pub enum DropRef {
        Pattern(Index<Pattern>),
        PatternChain(Index<Chain<Pattern>>),
        TimedStep(Index<TimedStep>),
        TimedStepChain(Index<Chain<TimedStep>>),
    }
}

use private::DropRef;

/// Helper function to combine multiple iterators of different types.
fn combine_iters<T>(
    opt_iter1: Option<impl IntoIterator<Item = T>>,
    opt_iter2: Option<impl IntoIterator<Item = T>>,
    opt_iter3: Option<impl IntoIterator<Item = T>>,
    opt_iter4: Option<impl IntoIterator<Item = T>>,
) -> impl Iterator<Item = T> {
    opt_iter1
        .into_iter()
        .flatten()
        .chain(opt_iter2.into_iter().flatten())
        .chain(opt_iter3.into_iter().flatten())
        .chain(opt_iter4.into_iter().flatten())
}

fn get_drop_refs<'a, T: ArenaItem>(
    multiple: Multiple<T>,
    arena: &'a impl Arena<Chain<T>>,
    make_ref: impl Fn(Index<Chain<T>>) -> DropRef + 'a,
) -> impl Iterator<Item = ArenaResult<DropRef>> + 'a {
    multiple
        .iter_with_chain(arena)
        .map(move |result| result.map(&make_ref))
}

fn process_drop_ref<'a>(
    reference: DropRef,
    arenas: &'a impl PatternArenas,
) -> ArenaResult<impl Iterator<Item = ArenaResult<DropRef>> + 'a> {
    let pattern_index_to_iter = |pattern_index: Index<Pattern>| {
        once(pattern_index)
            .map(DropRef::Pattern)
            .map(Ok)
    };
    let timed_step_index_to_iter = |pattern_index: Index<TimedStep>| {
        once(pattern_index)
            .map(DropRef::TimedStep)
            .map(Ok)
    };

    let pattern_drop_refs = |pattern: Pattern| match pattern {
        Pattern::Cat(multiple)
        | Pattern::Seq(multiple)
        | Pattern::Stack(multiple) => {
            let iter = get_drop_refs(
                multiple,
                arenas.get_pattern_chain_arena(),
                DropRef::PatternChain,
            );
            combine_iters(Some(iter), None, None, None)
        }
        Pattern::TimeCat(multiple) | Pattern::Arrange(multiple) => {
            let iter = get_drop_refs(
                multiple,
                arenas.get_timed_step_chain_arena(),
                DropRef::TimedStepChain,
            );
            combine_iters(None, Some(iter), None, None)
        }
        Pattern::Note(_) | Pattern::Silence => {
            combine_iters(None, None, None, None)
        }
    };

    match reference {
        DropRef::Pattern(index) => {
            let pattern = arenas.get_pattern_arena().take(index)?;
            Ok(pattern_drop_refs(pattern))
        }
        DropRef::TimedStep(index) => {
            let TimedStep(_, pattern_index) = arenas
                .get_timed_step_arena()
                .take(index)?;
            let iter = pattern_index_to_iter(pattern_index);
            Ok(combine_iters(None, None, Some(iter), None))
        }
        DropRef::PatternChain(index) => {
            let pattern_index = arenas
                .get_pattern_chain_arena()
                .take(index)?
                .0;
            let iter = pattern_index_to_iter(pattern_index);
            Ok(combine_iters(None, None, Some(iter), None))
        }
        DropRef::TimedStepChain(index) => {
            let timed_step_index = arenas
                .get_timed_step_chain_arena()
                .take(index)?
                .0;
            let iter = timed_step_index_to_iter(timed_step_index);
            Ok(combine_iters(None, None, None, Some(iter)))
        }
    }
}

impl<Arenas: PatternArenas> DropRefs<Arenas> for DropRef {
    type Reference = DropRef;

    fn start_ref(drop_ref: DropRef) -> Self::Reference {
        drop_ref
    }

    fn process_ref<'a>(
        reference: Self::Reference,
        arenas: &'a Arenas,
    ) -> ArenaResult<impl Iterator<Item = ArenaResult<Self::Reference>> + 'a>
    {
        process_drop_ref(reference, arenas)
    }
}
