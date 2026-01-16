use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;

#[derive(Debug)]
pub struct PatternDropAdapter<'a, Arenas: PatternArenas> {
    index: Option<Index<Pattern>>,
    arenas: &'a Arenas,
}

pub struct PatternChainDropAdapter<'a, Arenas: PatternArenas> {
    index: Option<Index<Chain<Pattern>>>,
    arenas: &'a Arenas,
}

pub struct TimedStepDropAdapter<'a, Arenas: PatternArenas> {
    index: Option<Index<TimedStep>>,
    arenas: &'a Arenas,
}

pub struct TimedStepChainDropAdapter<'a, Arenas: PatternArenas> {
    index: Option<Index<Chain<TimedStep>>>,
    arenas: &'a Arenas,
}

pub trait DropAdapter<'a, Item, Arenas> {
    fn new(index: Index<Item>, arenas: &'a Arenas) -> Self;

    fn take_index(&mut self) -> Index<Item>;
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Pattern, Arenas>
    for PatternDropAdapter<'a, Arenas>
{
    fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self { index: Some(index), arenas }
    }

    fn take_index(&mut self) -> Index<Pattern> {
        self.index.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Chain<Pattern>, Arenas>
    for PatternChainDropAdapter<'a, Arenas>
{
    fn new(index: Index<Chain<Pattern>>, arenas: &'a Arenas) -> Self {
        Self { index: Some(index), arenas }
    }

    fn take_index(&mut self) -> Index<Chain<Pattern>> {
        self.index.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, TimedStep, Arenas>
    for TimedStepDropAdapter<'a, Arenas>
{
    fn new(index: Index<TimedStep>, arenas: &'a Arenas) -> Self {
        Self { index: Some(index), arenas }
    }

    fn take_index(&mut self) -> Index<TimedStep> {
        self.index.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> DropAdapter<'a, Chain<TimedStep>, Arenas>
    for TimedStepChainDropAdapter<'a, Arenas>
{
    fn new(index: Index<Chain<TimedStep>>, arenas: &'a Arenas) -> Self {
        Self { index: Some(index), arenas }
    }

    fn take_index(&mut self) -> Index<Chain<TimedStep>> {
        self.index.take().unwrap()
    }
}

impl<'a, Arenas: PatternArenas> Drop for PatternDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.index.take() {
            visit_pattern(&PatternDropVisitor { arenas: self.arenas }, index);
        }
    }
}

impl<'a, Arenas: PatternArenas> Drop for PatternChainDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        let drop_pattern = |pattern_index, arenas: &Arenas| {
            visit_pattern(&PatternDropVisitor { arenas }, pattern_index)
        };
        if let Some(index) = self.index.take() {
            chain_drop(
                index,
                self.arenas.get_pattern_chain_arena(),
                self.arenas,
                drop_pattern,
            );
        }
    }
}

fn drop_timed_step(
    timed_step_index: Index<TimedStep>,
    arenas: &impl PatternArenas,
) {
    arenas
        .get_timed_step_arena()
        .take(timed_step_index)
        .unwrap();
}

impl<'a, Arenas: PatternArenas> Drop for TimedStepDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.index.take() {
            drop_timed_step(index, self.arenas);
        }
    }
}

impl<'a, Arenas: PatternArenas> Drop for TimedStepChainDropAdapter<'a, Arenas> {
    fn drop(&mut self) {
        if let Some(index) = self.index.take() {
            chain_drop(
                index,
                self.arenas.get_timed_step_chain_arena(),
                self.arenas,
                drop_timed_step,
            );
        }
    }
}

/// Helper function to drop a [Chain].
pub fn chain_drop<Item: ArenaItem, Arenas: PatternArenas>(
    chain_index: Index<Chain<Item>>,
    chain_arena: &impl Arena<Chain<Item>>,
    arenas: &Arenas,
    drop_item: impl Fn(Index<Item>, &Arenas),
) {
    let mut cur_index = chain_index;
    loop {
        let chain = chain_arena
            .take(cur_index.clone())
            .unwrap();
        match chain {
            Chain::Nil => break,
            Chain::Cons { head, tail } => {
                drop_item(head, arenas);
                cur_index = tail;
            }
        }
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
    type PatternChainEntry = ();
    type PatternChainOutput = ();
    type TimedStepOutput = ();
    type TimedStepChainEntry = ();
    type TimedStepChainOutput = ();
    const CHAINS_FOLD_RIGHT: bool = false;

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

    fn exit_cat(
        &self,
        chain_index: Index<Chain<Pattern>>,
        _: Self::PatternChainEntry,
        _: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        self.arenas
            .get_pattern_chain_arena()
            .take(chain_index)
            .unwrap();
    }

    fn exit_seq(
        &self,
        chain_index: Index<Chain<Pattern>>,
        _: Self::PatternChainEntry,
        _: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        self.arenas
            .get_pattern_chain_arena()
            .take(chain_index)
            .unwrap();
    }

    fn exit_stack(
        &self,
        chain_index: Index<Chain<Pattern>>,
        _: Self::PatternChainEntry,
        _: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        self.arenas
            .get_pattern_chain_arena()
            .take(chain_index)
            .unwrap();
    }

    fn exit_time_cat(
        &self,
        chain_index: Index<Chain<TimedStep>>,
        _: Self::TimedStepChainEntry,
        _: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput {
        self.arenas
            .get_timed_step_chain_arena()
            .take(chain_index)
            .unwrap();
    }

    fn map_note_unit(&self, _: NoteUnit) -> Self::PatternOutput {}

    fn map_silence(&self) -> Self::PatternOutput {}

    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput {}

    fn fold_pattern_chain_output(
        &self,
        _: Self::PatternChainOutput,
        next_chain_index: Index<Chain<Pattern>>,
        pattern_index: Index<Pattern>,
    ) -> Self::PatternChainOutput {
        // Drop at the pattern index.
        visit_pattern(self, pattern_index);
        self.arenas
            .get_pattern_chain_arena()
            .take(next_chain_index)
            .unwrap();
    }

    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput {}

    fn fold_timed_step_chain_output(
        &self,
        _: Self::TimedStepChainOutput,
        next_chain_index: Index<Chain<TimedStep>>,
        timed_step_index: Index<TimedStep>,
    ) -> Self::TimedStepChainOutput {
        drop_timed_step(timed_step_index, self.arenas);
        self.arenas
            .get_timed_step_chain_arena()
            .take(next_chain_index)
            .unwrap();
    }

    fn enter_cat(
        &self,
        _index: Index<Chain<Pattern>>,
    ) -> Self::PatternChainEntry {
    }

    fn enter_seq(
        &self,
        _index: Index<Chain<Pattern>>,
    ) -> Self::PatternChainEntry {
    }

    fn enter_stack(
        &self,
        _index: Index<Chain<Pattern>>,
    ) -> Self::PatternChainEntry {
    }

    fn enter_time_cat(
        &self,
        _index: Index<Chain<TimedStep>>,
    ) -> Self::PatternChainEntry {
    }
}
