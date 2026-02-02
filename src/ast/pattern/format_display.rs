use core::cell::RefCell;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::ops::DerefMut;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::structures::chain::Chain;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;

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
        visit_pattern(&visitor, self.index.clone())
    }
}

fn write_str<'a, 'b, Arenas>(
    visitor: &PatternDisplayVisitor<'a, 'b, Arenas>,
    str: &'static str,
) -> core::fmt::Result {
    let mut formatter = visitor.formatter.borrow_mut();
    write!(formatter, "{str}")
}

fn print_multiple<Item: ArenaItem>(
    multiple: Multiple<Item>,
    chain_arena: &impl Arena<Chain<Item>>,
    formatter: &RefCell<&mut Formatter<'_>>,
    mut display_item: impl FnMut(Index<Item>) -> fmt::Result,
) -> core::fmt::Result {
    let length = multiple.length();
    let mut i = 1;
    multiple.fold_left(chain_arena, Ok(()), |output, index| {
        // Check if we are the last value in the chain.
        let not_at_end = i != length;
        i += 1;
        let separator = if not_at_end { ", " } else { "" };
        // Print the separator, wherever applicable.
        let write_separator = |()| {
            formatter
                .borrow_mut()
                .write_str(separator)
        };
        output
            .and_then(|()| display_item(index))
            .and_then(write_separator)
    })
}

fn print_multiple_pattern(
    multiple: Multiple<Pattern>,
    arenas: &impl PatternArenas,
    formatter: &mut Formatter<'_>,
) -> core::fmt::Result {
    let formatter_refcell = RefCell::new(formatter);
    print_multiple(
        multiple,
        arenas.get_pattern_chain_arena(),
        &formatter_refcell,
        |pattern_index| {
            let mut borrowed_formatter = formatter_refcell.borrow_mut();
            let visitor = PatternDisplayVisitor {
                arenas,
                formatter: RefCell::new(borrowed_formatter.deref_mut()),
            };
            visit_pattern(&visitor, pattern_index)
        },
    )
}

fn print_multiple_timed_step(
    multiple: Multiple<TimedStep>,
    arenas: &impl PatternArenas,
    formatter: &mut Formatter<'_>,
) -> core::fmt::Result {
    let formatter_refcell = RefCell::new(formatter);
    print_multiple(
        multiple,
        arenas.get_timed_step_chain_arena(),
        &formatter_refcell,
        |timed_step_index| {
            let cloned_timed_step = arenas
                .get_timed_step_arena()
                .inspect(timed_step_index, Clone::clone)
                .unwrap();
            let TimedStep(time_unit, pattern_index) = cloned_timed_step;
            let print_pattern = |formatter: &mut Formatter<'_>| {
                let visitor = PatternDisplayVisitor {
                    arenas,
                    formatter: RefCell::new(formatter),
                };
                visit_pattern(&visitor, pattern_index)
            };
            let mut borrowed_formatter = formatter_refcell.borrow_mut();
            write!(borrowed_formatter, "[{time_unit:?}, ")?;
            print_pattern(&mut borrowed_formatter)?;
            write!(borrowed_formatter, "]")?;
            Ok(())
        },
    )
}

impl<'a, 'b, Arenas: PatternArenas> PatternVisitor
    for PatternDisplayVisitor<'a, 'b, Arenas>
{
    type Output = core::fmt::Result;
    type PatternOutput = core::fmt::Result;

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

    fn map_cat(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        let print_patterns = |()| {
            print_multiple_pattern(
                multiple,
                self.arenas,
                self.formatter.borrow_mut().deref_mut(),
            )
        };
        write_str(self, "Cat(")
            .and_then(print_patterns)
            .and_then(|()| write_str(self, ")"))
    }

    fn map_seq(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        let print_patterns = |()| {
            print_multiple_pattern(
                multiple,
                self.arenas,
                self.formatter.borrow_mut().deref_mut(),
            )
        };
        write_str(self, "Seq(")
            .and_then(print_patterns)
            .and_then(|()| write_str(self, ")"))
    }

    fn map_stack(&self, multiple: Multiple<Pattern>) -> Self::PatternOutput {
        let print_patterns = |()| {
            print_multiple_pattern(
                multiple,
                self.arenas,
                self.formatter.borrow_mut().deref_mut(),
            )
        };
        write_str(self, "Stack(")
            .and_then(print_patterns)
            .and_then(|()| write_str(self, ")"))
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        let print_timed_steps = |()| {
            print_multiple_timed_step(
                multiple,
                self.arenas,
                self.formatter.borrow_mut().deref_mut(),
            )
        };
        write_str(self, "TimeCat(")
            .and_then(print_timed_steps)
            .and_then(|()| write_str(self, ")"))
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
}
