use core::fmt::Debug;
use core::marker::PhantomData;

use crate::alloc_types::String;
use crate::alloc_types::format;
use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;

trait ChainFoldRightStrategy {
    const CHAINS_FOLD_RIGHT: bool;
}

struct ChainFoldRight;
impl ChainFoldRightStrategy for ChainFoldRight {
    const CHAINS_FOLD_RIGHT: bool = true;
}
struct ChainFoldLeft;
impl ChainFoldRightStrategy for ChainFoldLeft {
    const CHAINS_FOLD_RIGHT: bool = false;
}

struct PatternDisplayVisitor<
    'a,
    Arenas: PatternArenas,
    FoldStrategy: ChainFoldRightStrategy,
> {
    arenas: &'a Arenas,
    phantom: PhantomData<FoldStrategy>,
}

pub struct PatternDisplayVisitorR<'a, Arenas: PatternArenas> {
    inner: PatternDisplayVisitor<'a, Arenas, ChainFoldRight>,
}
pub struct PatternDisplayVisitorL<'a, Arenas: PatternArenas> {
    inner: PatternDisplayVisitor<'a, Arenas, ChainFoldLeft>,
}

impl<'a, Arenas: PatternArenas> PatternDisplayVisitorR<'a, Arenas> {
    #[allow(unused)]
    pub fn new(arenas: &'a Arenas) -> Self {
        Self { inner: PatternDisplayVisitor { arenas, phantom: PhantomData } }
    }

    #[allow(unused)]
    pub fn display(&self, pattern_index: Index<super::Pattern>) -> String {
        visit_pattern(&self.inner, pattern_index)
    }
}

impl<'a, Arenas: PatternArenas> PatternDisplayVisitorL<'a, Arenas> {
    #[cfg(test)]
    pub fn new(arenas: &'a Arenas) -> Self {
        Self { inner: PatternDisplayVisitor { arenas, phantom: PhantomData } }
    }

    #[allow(unused)]
    pub fn display(&self, pattern_index: Index<super::Pattern>) -> String {
        visit_pattern(&self.inner, pattern_index)
    }
}

fn fold_display_output<
    Item: ArenaItem + Debug,
    FoldStrategy: ChainFoldRightStrategy,
>(
    chain_output: String,
    displayed_item: String,
) -> String {
    // Check if we are the last value in the chain.
    let not_at_end = !chain_output.is_empty();
    let separator = if not_at_end { ", " } else { "" };
    if FoldStrategy::CHAINS_FOLD_RIGHT {
        format!("{displayed_item}{separator}{chain_output}")
    } else {
        format!("{chain_output}{separator}{displayed_item}")
    }
}

impl<'a, Arenas: PatternArenas, FoldStrategy: ChainFoldRightStrategy>
    PatternVisitor for PatternDisplayVisitor<'a, Arenas, FoldStrategy>
{
    type Output = String;
    type PatternOutput = String;
    type PatternChainOutput = String;
    type TimedStepOutput = String;
    type TimedStepChainOutput = String;

    const CHAINS_FOLD_RIGHT: bool = FoldStrategy::CHAINS_FOLD_RIGHT;

    fn get_arenas(&self) -> &impl PatternArenas {
        self.arenas
    }

    fn map_pattern(
        &self,
        _pattern_index: Index<super::Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output {
        pattern_output
    }

    fn map_cat(
        &self,
        _chain_index: Index<Chain<super::Pattern>>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        format!("Cat({pattern_chain_output})")
    }

    fn map_seq(
        &self,
        _chain_index: Index<Chain<super::Pattern>>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        format!("Seq({pattern_chain_output})")
    }

    fn map_stack(
        &self,
        _chain_index: Index<Chain<super::Pattern>>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        format!("Stack({pattern_chain_output})")
    }

    fn map_time_cat(
        &self,
        _chain_index: Index<Chain<super::TimedStep>>,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput {
        format!("TimeCat({timed_step_chain_output})")
    }

    fn map_note_unit(
        &self,
        unit: super::note::NoteUnit,
    ) -> Self::PatternOutput {
        format!("Unit({unit:?})")
    }

    fn map_silence(&self) -> Self::PatternOutput {
        String::from("Silence")
    }

    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput {
        String::new()
    }

    fn fold_pattern_chain_output(
        &self,
        chain_output: Self::PatternChainOutput,
        _tail_index: Index<Chain<super::Pattern>>,
        pattern_index: Index<super::Pattern>,
    ) -> Self::PatternChainOutput {
        fold_display_output::<super::Pattern, FoldStrategy>(
            chain_output,
            visit_pattern(self, pattern_index),
        )
    }

    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput {
        String::new()
    }

    fn fold_timed_step_chain_output(
        &self,
        chain_output: Self::TimedStepChainOutput,
        _tail_index: Index<Chain<super::TimedStep>>,
        timed_step_index: Index<super::TimedStep>,
    ) -> Self::TimedStepChainOutput {
        let displayed_timed_step = self
            .arenas
            .get_timed_step_arena()
            .inspect(timed_step_index, |timed_step| format!("{timed_step:?}"))
            .unwrap();
        fold_display_output::<super::Pattern, FoldStrategy>(
            chain_output,
            displayed_timed_step,
        )
    }
}
