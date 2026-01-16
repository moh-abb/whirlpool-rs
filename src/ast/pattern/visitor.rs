use core::fmt::Debug;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;

#[allow(unused)]
pub trait PatternVisitor {
    type Output;
    type PatternOutput;
    type PatternChainEntry;
    type PatternChainOutput;
    type TimedStepOutput;
    type TimedStepChainEntry;
    type TimedStepChainOutput;

    const CHAINS_FOLD_RIGHT: bool;

    fn get_arenas(&self) -> &impl PatternArenas;

    fn map_pattern(
        &self,
        pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output;

    fn enter_cat(&self, multiple: Multiple<Pattern>)
    -> Self::PatternChainEntry;
    fn enter_seq(&self, multiple: Multiple<Pattern>)
    -> Self::PatternChainEntry;
    fn enter_stack(
        &self,
        multiple: Multiple<Pattern>,
    ) -> Self::PatternChainEntry;
    fn enter_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::TimedStepChainEntry;

    fn exit_cat(
        &self,
        multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput;
    fn exit_seq(
        &self,
        multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput;
    fn exit_stack(
        &self,
        multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput;
    fn exit_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
        output_from_entry: Self::TimedStepChainEntry,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput;

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput;
    fn map_silence(&self) -> Self::PatternOutput;

    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput;
    fn fold_pattern_chain_output(
        &self,
        pattern_chain_output: Self::PatternChainOutput,
        tail_index: Index<Chain<Pattern>>,
        pattern_index: Index<Pattern>,
    ) -> Self::PatternChainOutput;

    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput;
    fn fold_timed_step_chain_output(
        &self,
        timed_step_chain_output: Self::TimedStepChainOutput,
        tail_index: Index<Chain<TimedStep>>,
        timed_step_index: Index<TimedStep>,
    ) -> Self::TimedStepChainOutput;
}

type FoldOutputFn<Item, ChainOutput, State> =
    fn(&State, ChainOutput, Index<Chain<Item>>, Index<Item>) -> ChainOutput;

#[allow(unused)]
pub fn visit_pattern<V: PatternVisitor>(
    visitor: &V,
    pattern_index: Index<Pattern>,
) -> V::Output {
    let panic_with_err = |err| {
        panic!(
            "[visit_pattern]: accessing pattern at {pattern_index:?} gave error {err:?}",
        )
    };
    let cloned_pattern = visitor
        .get_arenas()
        .get_pattern_arena()
        .inspect(pattern_index.clone(), Clone::clone)
        .unwrap_or_else(panic_with_err);
    let result = match cloned_pattern.clone() {
        Pattern::Cat(multiple)
        | Pattern::Seq(multiple)
        | Pattern::Stack(multiple) => {
            let enter_func = match &cloned_pattern {
                Pattern::Cat(_) => V::enter_cat,
                Pattern::Seq(_) => V::enter_seq,
                Pattern::Stack(_) => V::enter_stack,
                _ => unreachable!(),
            };
            let pattern_chain_entry = enter_func(visitor, multiple.clone());
            let chain_fold = if V::CHAINS_FOLD_RIGHT {
                chain_fold_right
            } else {
                chain_fold_left
            };
            let new_chain_output = visitor.new_pattern_chain_output();
            let index = multiple.index.clone();
            let chain_output = chain_fold(
                visitor,
                index,
                visitor
                    .get_arenas()
                    .get_pattern_chain_arena(),
                new_chain_output,
                V::fold_pattern_chain_output,
            );
            let exit_func = match &cloned_pattern {
                Pattern::Cat(_) => V::exit_cat,
                Pattern::Seq(_) => V::exit_seq,
                Pattern::Stack(_) => V::exit_stack,
                _ => unreachable!(),
            };
            exit_func(visitor, multiple, pattern_chain_entry, chain_output)
        }
        Pattern::TimeCat(multiple) => {
            let timed_step_chain_entry =
                visitor.enter_time_cat(multiple.clone());
            let chain_fold = if V::CHAINS_FOLD_RIGHT {
                chain_fold_right
            } else {
                chain_fold_left
            };
            let new_chain_output = visitor.new_timed_step_chain_output();
            let timed_step_chain_output = chain_fold(
                visitor,
                multiple.index.clone(),
                visitor
                    .get_arenas()
                    .get_timed_step_chain_arena(),
                new_chain_output,
                V::fold_timed_step_chain_output,
            );
            visitor.exit_time_cat(
                multiple,
                timed_step_chain_entry,
                timed_step_chain_output,
            )
        }
        Pattern::Note(note_unit) => visitor.map_note_unit(note_unit),
        Pattern::Silence => visitor.map_silence(),
    };
    visitor.map_pattern(pattern_index, result)
}

fn chain_fold_left<Item: ArenaItem + Debug + Clone, ChainOutput, State>(
    state: &State,
    chain_index: Index<Chain<Item>>,
    item_chain_arena: &impl Arena<Chain<Item>>,
    initial_output: ChainOutput,
    fold_output: FoldOutputFn<Item, ChainOutput, State>,
) -> ChainOutput {
    let panic_with_err = |index: &Index<Chain<Item>>, err| {
        panic!(
            "[chain_fold_left]: accessing chain at {index:?} gave error {err:?}",
        );
    };
    let get_chain = |index: &Index<Chain<Item>>| {
        item_chain_arena
            .inspect(index.clone(), Chain::clone)
            .unwrap_or_else(|err| panic_with_err(index, err))
    };
    let mut cur_chain = get_chain(&chain_index);
    let mut fold_result = initial_output;
    loop {
        let Chain::Cons { head, tail } = cur_chain else { break };
        // Save the tail value before iterating.
        let tail_chain = get_chain(&tail);
        fold_result = fold_output(state, fold_result, tail.clone(), head);
        // Continue invariant
        cur_chain = tail_chain;
    }
    fold_result
}

fn chain_fold_right<Item: ArenaItem + Debug + Clone, ChainOutput, State>(
    state: &State,
    chain_index: Index<Chain<Item>>,
    item_chain_arena: &impl Arena<Chain<Item>>,
    initial_output: ChainOutput,
    fold_output: FoldOutputFn<Item, ChainOutput, State>,
) -> ChainOutput {
    let panic_with_err = |err| {
        panic!(
            "[chain_fold_right]: accessing chain at {chain_index:?} gave error {err:?}",
        )
    };
    let cloned_chain = item_chain_arena
        .inspect(chain_index.clone(), Chain::clone)
        .unwrap_or_else(panic_with_err);
    let Chain::Cons { head, tail } = cloned_chain else {
        return initial_output;
    };
    let tail_result = chain_fold_right(
        state,
        tail.clone(),
        item_chain_arena,
        initial_output,
        fold_output,
    );
    let fold_result = fold_output(state, tail_result, tail.clone(), head);
    fold_result
}
