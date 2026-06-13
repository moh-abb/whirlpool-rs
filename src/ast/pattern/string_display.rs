use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::String;
use crate::mem::format;

pub struct PatternDisplayVisitor<'a, Arenas: PatternArenas> {
    arenas: &'a Arenas,
    folds_right: bool,
}

impl<'a, Arenas: PatternArenas> PatternDisplayVisitor<'a, Arenas> {
    #[allow(unused)]
    pub fn new_right(arenas: &'a Arenas) -> Self {
        Self { arenas, folds_right: true }
    }

    #[allow(unused)]
    pub fn new_left(arenas: &'a Arenas) -> Self {
        Self { arenas, folds_right: false }
    }

    #[allow(unused)]
    pub fn display(
        &self,
        pattern_index: Index<super::Pattern>,
    ) -> ArenaResult<String> {
        visit_pattern(self, pattern_index).flatten()
    }

    fn try_fold(
        &self,
        mut iter: impl DoubleEndedIterator<Item = ArenaResult<String>>,
    ) -> ArenaResult<String> {
        let fold_func = if self.folds_right {
            DoubleEndedIterator::try_rfold
        } else {
            Iterator::try_fold
        };
        fold_func(&mut iter, String::new(), |acc: String, opt_x| {
            // Check if we are the last value in the chain.
            let not_at_end = !acc.is_empty();
            let separator = if not_at_end { ", " } else { "" };
            let x = opt_x?;
            let formatted = if self.folds_right {
                format!("{x}{separator}{acc}")
            } else {
                format!("{acc}{separator}{x}")
            };
            Ok(formatted)
        })
    }

    fn print_multiple_pattern(
        &self,
        multiple: Multiple<Pattern>,
    ) -> ArenaResult<String> {
        let iter = multiple
            .checked_iter(self.arenas.get_pattern_chain_arena())
            .map(|x| self.display(x?));
        self.try_fold(iter)
    }

    fn print_multiple_timed_step(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> ArenaResult<String> {
        let iter = multiple
            .checked_iter(self.arenas.get_timed_step_chain_arena())
            .map(|x| {
                self.arenas
                    .get_timed_step_arena()
                    .map(x?, Clone::clone)
            })
            .map(|x| {
                let TimedStep(time_unit, pattern) = x?;
                let displayed_pattern = self.display(pattern)?;
                let formatted = format!("[{time_unit:?}, {displayed_pattern}]");
                Ok(formatted)
            });
        self.try_fold(iter)
    }
}

impl<'a, Arenas: PatternArenas> PatternVisitor
    for PatternDisplayVisitor<'a, Arenas>
{
    type Output = ArenaResult<String>;
    type PatternOutput = ArenaResult<String>;

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
        multiple: Multiple<super::Pattern>,
    ) -> Self::PatternOutput {
        Ok(format!("Cat({})", self.print_multiple_pattern(multiple)?))
    }

    fn map_seq(
        &self,
        multiple: Multiple<super::Pattern>,
    ) -> Self::PatternOutput {
        Ok(format!("Seq({})", self.print_multiple_pattern(multiple)?))
    }

    fn map_stack(
        &self,
        multiple: Multiple<super::Pattern>,
    ) -> Self::PatternOutput {
        Ok(format!("Stack({})", self.print_multiple_pattern(multiple)?))
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        Ok(format!("TimeCat({})", self.print_multiple_timed_step(multiple)?))
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        Ok(format!("Arrange({})", self.print_multiple_timed_step(multiple)?))
    }

    fn map_note_unit(
        &self,
        unit: super::note::NoteUnit,
    ) -> Self::PatternOutput {
        Ok(format!("Unit({unit:?})"))
    }

    fn map_silence(&self) -> Self::PatternOutput {
        Ok(String::from("Silence"))
    }
}
