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
    type PatternChainOutput;
    type TimedStepOutput;
    type TimedStepChainOutput;

    const CHAINS_FOLD_RIGHT: bool;

    fn get_arenas(&self) -> &impl PatternArenas;

    fn map_pattern(
        &self,
        pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output;
    fn map_cat(
        &self,
        multiple: Multiple<Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput;
    fn map_seq(
        &self,
        multiple: Multiple<Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput;
    fn map_stack(
        &self,
        multiple: Multiple<Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput;
    fn map_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
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

pub trait EnteringPatternVisitor: PatternVisitor {
    type PatternChainEntry;
    type TimedStepChainEntry;

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
    ) -> Self::PatternChainOutput;
    fn exit_seq(
        &self,
        multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternChainOutput;
    fn exit_stack(
        &self,
        multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternChainOutput;
    fn exit_time_cat(
        &self,
        multiple: Multiple<TimedStep>,
        output_from_entry: Self::TimedStepChainEntry,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::TimedStepChainOutput;
}

type FoldOutputFn<Item, ChainOutput, State> =
    fn(&State, ChainOutput, Index<Chain<Item>>, Index<Item>) -> ChainOutput;

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

fn get_chain_output<
    Item: ArenaItem + Clone + Debug,
    V: PatternVisitor,
    ChainOutput,
>(
    visitor: &V,
    item_chain_arena: &impl Arena<Chain<Item>>,
    multiple: Multiple<Item>,
    new_chain_output: fn(&V) -> ChainOutput,
    fold_output: FoldOutputFn<Item, ChainOutput, V>,
) -> ChainOutput {
    let chain_fold =
        if V::CHAINS_FOLD_RIGHT { chain_fold_right } else { chain_fold_left };
    let new_chain_output = new_chain_output(visitor);
    let index = multiple.index.clone();
    chain_fold(visitor, index, item_chain_arena, new_chain_output, fold_output)
}

type EnterMultipleFn<V, Item, Entry> = fn(&V, Multiple<Item>) -> Entry;
type ExitMultipleFn<V, Item, Entry, Output> =
    fn(&V, Multiple<Item>, Entry, Output) -> Output;
type EnterAndExitFn<V, Item, Entry, Output> = Option<(
    EnterMultipleFn<V, Item, Entry>,
    ExitMultipleFn<V, Item, Entry, Output>,
)>;

fn visit_pattern_with_optional_entering<
    V: PatternVisitor,
    PatternEntry,
    TimedStepEntry,
>(
    visitor: &V,
    pattern_index: Index<Pattern>,
    enter_and_exit_multiple_patterns: impl Fn(
        &Pattern,
    ) -> EnterAndExitFn<
        V,
        Pattern,
        PatternEntry,
        V::PatternChainOutput,
    >,
    enter_and_exit_multiple_timed_steps: EnterAndExitFn<
        V,
        TimedStep,
        TimedStepEntry,
        V::TimedStepChainOutput,
    >,
) -> V::Output {
    let cloned_pattern = get_cloned_pattern(visitor, pattern_index.clone());
    let pattern_chain_arena = visitor
        .get_arenas()
        .get_pattern_chain_arena();
    let timed_step_chain_arena = visitor
        .get_arenas()
        .get_timed_step_chain_arena();
    let get_multiple_pattern_output = |multiple: &Multiple<Pattern>| {
        get_chain_output(
            visitor,
            pattern_chain_arena,
            multiple.clone(),
            V::new_pattern_chain_output,
            V::fold_pattern_chain_output,
        )
    };
    let get_multiple_timed_step_output = |multiple: &Multiple<TimedStep>| {
        get_chain_output(
            visitor,
            timed_step_chain_arena,
            multiple.clone(),
            V::new_timed_step_chain_output,
            V::fold_timed_step_chain_output,
        )
    };
    let result = match cloned_pattern.clone() {
        Pattern::Cat(multiple)
        | Pattern::Seq(multiple)
        | Pattern::Stack(multiple) => {
            let output_with_entry_and_exit =
                match enter_and_exit_multiple_patterns(&cloned_pattern) {
                    None => get_multiple_pattern_output(&multiple),
                    Some((enter_func, exit_func)) => {
                        let entry = enter_func(visitor, multiple.clone());
                        let output = get_multiple_pattern_output(&multiple);
                        exit_func(visitor, multiple.clone(), entry, output)
                    }
                };
            let map_func = match &cloned_pattern {
                Pattern::Cat(_) => V::map_cat,
                Pattern::Seq(_) => V::map_seq,
                Pattern::Stack(_) => V::map_stack,
                _ => unreachable!(),
            };
            map_func(visitor, multiple, output_with_entry_and_exit)
        }
        Pattern::TimeCat(multiple) => {
            let output_with_entry_and_exit =
                match enter_and_exit_multiple_timed_steps {
                    None => get_multiple_timed_step_output(&multiple),
                    Some((enter_func, exit_func)) => {
                        let entry = enter_func(visitor, multiple.clone());
                        let output = get_multiple_timed_step_output(&multiple);
                        exit_func(visitor, multiple.clone(), entry, output)
                    }
                };
            let map_func = V::map_time_cat;
            map_func(visitor, multiple, output_with_entry_and_exit)
        }
        Pattern::Note(note_unit) => visitor.map_note_unit(note_unit),
        Pattern::Silence => visitor.map_silence(),
    };
    visitor.map_pattern(pattern_index, result)
}

#[inline]
pub fn visit_pattern_with_entering<V: EnteringPatternVisitor>(
    visitor: &V,
    pattern_index: Index<Pattern>,
) -> V::Output {
    let enter_pattern = |pattern: &Pattern| match pattern {
        Pattern::Cat(_) => V::enter_cat,
        Pattern::Seq(_) => V::enter_seq,
        Pattern::Stack(_) => V::enter_stack,
        _ => unreachable!(),
    };
    let exit_pattern = |pattern: &Pattern| match pattern {
        Pattern::Cat(_) => V::exit_cat,
        Pattern::Seq(_) => V::exit_seq,
        Pattern::Stack(_) => V::exit_stack,
        _ => unreachable!(),
    };
    visit_pattern_with_optional_entering(
        visitor,
        pattern_index,
        |pattern| Some((enter_pattern(pattern), exit_pattern(pattern))),
        Some((V::enter_time_cat, V::exit_time_cat)),
    )
}

#[inline]
pub fn visit_pattern<V: PatternVisitor>(
    visitor: &V,
    pattern_index: Index<Pattern>,
) -> V::Output {
    visit_pattern_with_optional_entering(
        visitor,
        pattern_index,
        |_: &Pattern| EnterAndExitFn::<V, _, (), V::PatternChainOutput>::None,
        EnterAndExitFn::<V, _, (), V::TimedStepChainOutput>::None,
    )
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
