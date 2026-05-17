use core::fmt::Debug;
use core::marker::PhantomData;

use crate::alloc_types::String;
use crate::alloc_types::format;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::mem::arena::Arena;
use crate::mem::arena::ArenaItem;
use crate::structures::chain::Chain;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;

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

fn fold_display_output<FoldStrategy: ChainFoldRightStrategy>(
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

fn print_multiple<
    Item: ArenaItem + Debug,
    FoldStrategy: ChainFoldRightStrategy,
>(
    multiple: Multiple<Item>,
    chain_arena: &impl Arena<Chain<Item>>,
    mut print_item: impl FnMut(Index<Item>) -> String,
) -> String {
    let chain_fold = if FoldStrategy::CHAINS_FOLD_RIGHT {
        Multiple::fold_right
    } else {
        Multiple::fold_left
    };
    chain_fold(&multiple, chain_arena, String::new(), |acc, x| {
        fold_display_output::<FoldStrategy>(acc, print_item(x))
    })
}

fn print_multiple_pattern<
    Arenas: PatternArenas,
    FoldStrategy: ChainFoldRightStrategy,
>(
    multiple: Multiple<Pattern>,
    arenas: &Arenas,
) -> String {
    print_multiple::<_, FoldStrategy>(
        multiple,
        arenas.get_pattern_chain_arena(),
        |pattern| {
            let visitor = PatternDisplayVisitor::<_, FoldStrategy> {
                arenas,
                phantom: PhantomData,
            };
            visit_pattern(&visitor, pattern)
        },
    )
}

fn print_multiple_timed_step<
    Arenas: PatternArenas,
    FoldStrategy: ChainFoldRightStrategy,
>(
    multiple: Multiple<TimedStep>,
    arenas: &Arenas,
) -> String {
    print_multiple::<_, FoldStrategy>(
        multiple,
        arenas.get_timed_step_chain_arena(),
        |timed_step| {
            let cloned_timed_step = arenas
                .get_timed_step_arena()
                .inspect(timed_step, Clone::clone)
                .unwrap();
            let TimedStep(time_unit, pattern) = cloned_timed_step;
            let visitor = PatternDisplayVisitor::<_, FoldStrategy> {
                arenas,
                phantom: PhantomData,
            };
            let displayed_pattern = visit_pattern(&visitor, pattern);
            format!("[{time_unit:?}, {displayed_pattern}]")
        },
    )
}

impl<'a, Arenas: PatternArenas, FoldStrategy: ChainFoldRightStrategy>
    PatternVisitor for PatternDisplayVisitor<'a, Arenas, FoldStrategy>
{
    type Output = String;
    type PatternOutput = String;

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
        let displayed_multiple =
            print_multiple_pattern::<_, FoldStrategy>(multiple, self.arenas);
        format!("Cat({displayed_multiple})")
    }

    fn map_seq(
        &self,
        multiple: Multiple<super::Pattern>,
    ) -> Self::PatternOutput {
        let displayed_multiple =
            print_multiple_pattern::<_, FoldStrategy>(multiple, self.arenas);
        format!("Seq({displayed_multiple})")
    }

    fn map_stack(
        &self,
        multiple: Multiple<super::Pattern>,
    ) -> Self::PatternOutput {
        let displayed_multiple =
            print_multiple_pattern::<_, FoldStrategy>(multiple, self.arenas);
        format!("Stack({displayed_multiple})")
    }

    fn map_time_cat(
        &self,
        multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        let displayed_multiple =
            print_multiple_timed_step::<_, FoldStrategy>(multiple, self.arenas);
        format!("TimeCat({displayed_multiple})")
    }

    fn map_arrange(
        &self,
        multiple: Multiple<TimedStep>,
    ) -> Self::PatternOutput {
        let displayed_multiple =
            print_multiple_timed_step::<_, FoldStrategy>(multiple, self.arenas);
        format!("Arrange({displayed_multiple})")
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
}
