use core::cell::RefCell;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::EnteringPatternVisitor;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern_with_entering;

pub struct PatternDisplayAdapter<'a, Arenas> {
    index: Index<Pattern>,
    arenas: &'a Arenas,
}

impl<'a, Arenas: PatternArenas> PatternDisplayAdapter<'a, Arenas> {
    #[allow(unused)]
    pub fn new(index: Index<Pattern>, arenas: &'a Arenas) -> Self {
        Self { index, arenas }
    }
}

struct PatternDisplayVisitor<'a, 'b, Arenas> {
    arenas: &'a Arenas,
    formatter: RefCell<&'a mut Formatter<'b>>,
}

impl<'a, Arenas: PatternArenas> Display for PatternDisplayAdapter<'a, Arenas> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let visitor = PatternDisplayVisitor {
            arenas: self.arenas,
            formatter: RefCell::new(f),
        };
        visit_pattern_with_entering(&visitor, self.index.clone())
    }
}

fn write_str<'a, 'b, Arenas>(
    visitor: &PatternDisplayVisitor<'a, 'b, Arenas>,
    str: &'static str,
) -> core::fmt::Result {
    let mut formatter = visitor.formatter.borrow_mut();
    write!(formatter, "{str}")
}

fn print_chain_item<
    'a,
    Item: ArenaItem,
    Arenas: PatternArenas,
    CA: Arena<Chain<Item>> + 'a,
>(
    visitor: &'a PatternDisplayVisitor<'_, '_, Arenas>,
    get_chain_arena: impl Fn(&'a Arenas) -> &'a CA,
    chain_output: fmt::Result,
    tail_index: Index<Chain<Item>>,
    display_item: impl FnOnce() -> fmt::Result,
) -> core::fmt::Result {
    // Check if we are the last value in the chain.
    let not_at_end = get_chain_arena(visitor.arenas)
        .inspect(tail_index, |chain| matches!(chain, Chain::Cons { .. }))
        .unwrap();
    let separator = if not_at_end { ", " } else { "" };
    // Print the separator, wherever applicable.
    let write_separator = |()| {
        let mut formatter = visitor.formatter.borrow_mut();
        formatter.write_str(separator)
    };
    chain_output
        .and_then(|()| display_item())
        .and_then(write_separator)
}

impl<'a, 'b, Arenas: PatternArenas> PatternVisitor
    for PatternDisplayVisitor<'a, 'b, Arenas>
{
    type Output = core::fmt::Result;
    type PatternOutput = core::fmt::Result;
    type PatternChainOutput = core::fmt::Result;
    type TimedStepOutput = core::fmt::Result;
    type TimedStepChainOutput = core::fmt::Result;

    const CHAINS_FOLD_RIGHT: bool = false;

    fn get_arenas(&self) -> &impl PatternArenas {
        self.arenas
    }

    fn map_pattern(
        &self,
        _pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output {
        pattern_output
    }

    fn map_cat(
        &self,
        _multiple: Multiple<super::Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        pattern_chain_output
    }

    fn map_seq(
        &self,
        _multiple: Multiple<super::Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        pattern_chain_output
    }

    fn map_stack(
        &self,
        _multiple: Multiple<super::Pattern>,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        pattern_chain_output
    }

    fn map_time_cat(
        &self,
        _multiple: Multiple<super::TimedStep>,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput {
        timed_step_chain_output
    }

    fn map_note_unit(
        &self,
        unit: super::note::NoteUnit,
    ) -> Self::PatternOutput {
        let mut formatter = self.formatter.borrow_mut();
        write!(formatter, "Unit({unit:?})")
    }

    fn map_silence(&self) -> Self::PatternOutput {
        let mut formatter = self.formatter.borrow_mut();
        write!(formatter, "Silence")
    }

    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput {
        Ok(())
    }

    fn fold_pattern_chain_output(
        &self,
        pattern_chain_output: Self::PatternChainOutput,
        tail_index: Index<Chain<Pattern>>,
        pattern_index: Index<Pattern>,
    ) -> Self::PatternChainOutput {
        print_chain_item(
            self,
            Arenas::get_pattern_chain_arena,
            pattern_chain_output,
            tail_index,
            || {
                let mut formatter = self.formatter.borrow_mut();
                let visitor = PatternDisplayVisitor {
                    arenas: self.arenas,
                    formatter: RefCell::new(&mut formatter),
                };
                visit_pattern_with_entering(&visitor, pattern_index)
            },
        )
    }

    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput {
        Ok(())
    }

    fn fold_timed_step_chain_output(
        &self,
        timed_step_chain_output: Self::TimedStepChainOutput,
        tail_index: Index<Chain<super::TimedStep>>,
        timed_step_index: Index<super::TimedStep>,
    ) -> Self::TimedStepChainOutput {
        print_chain_item(
            self,
            Arenas::get_timed_step_chain_arena,
            timed_step_chain_output,
            tail_index,
            || {
                let mut formatter = self.formatter.borrow_mut();
                self.arenas
                    .get_timed_step_arena()
                    .inspect(timed_step_index, |timed_step| {
                        write!(formatter, "{timed_step:?}")
                    })
                    .unwrap()
            },
        )
    }
}

impl<'a, 'b, Arenas: PatternArenas> EnteringPatternVisitor
    for PatternDisplayVisitor<'a, 'b, Arenas>
{
    type PatternChainEntry = core::fmt::Result;
    type TimedStepChainEntry = core::fmt::Result;

    fn enter_cat(
        &self,
        _multiple: Multiple<super::Pattern>,
    ) -> Self::PatternChainEntry {
        write_str(self, "Cat(")
    }

    fn enter_seq(
        &self,
        _multiple: Multiple<super::Pattern>,
    ) -> Self::PatternChainEntry {
        write_str(self, "Seq(")
    }

    fn enter_stack(
        &self,
        _multiple: Multiple<super::Pattern>,
    ) -> Self::PatternChainEntry {
        write_str(self, "Stack(")
    }

    fn enter_time_cat(
        &self,
        _multiple: Multiple<super::TimedStep>,
    ) -> Self::TimedStepChainEntry {
        write_str(self, "TimeCat(")
    }

    fn exit_cat(
        &self,
        _multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternChainOutput {
        output_from_entry
            .and(pattern_chain_output)
            .and_then(|()| write_str(self, ")"))
    }

    fn exit_seq(
        &self,
        _multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternChainOutput {
        output_from_entry
            .and(pattern_chain_output)
            .and_then(|()| write_str(self, ")"))
    }

    fn exit_stack(
        &self,
        _multiple: Multiple<Pattern>,
        output_from_entry: Self::PatternChainEntry,
        pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternChainOutput {
        output_from_entry
            .and(pattern_chain_output)
            .and_then(|()| write_str(self, ")"))
    }

    fn exit_time_cat(
        &self,
        _multiple: Multiple<super::TimedStep>,
        output_from_entry: Self::TimedStepChainEntry,
        timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::TimedStepChainOutput {
        output_from_entry
            .and(timed_step_chain_output)
            .and_then(|()| write_str(self, ")"))
    }
}
