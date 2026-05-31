use crate::ast::Pattern;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Chain;
use crate::mem::Index;
use crate::mem::Multiple;

pub enum PatternDropRef {
    Pattern(Index<Pattern>),
    PatternChain(Index<Chain<Pattern>>),
    TimedStep(Index<TimedStep>),
    TimedStepChain(Index<Chain<TimedStep>>),
}

/// Used to keep track of the current nodes in a [Pattern] while cloning.
/// Fields called "index" refer to the original (non-cloned) element unless
/// specified otherwise.
pub enum PatternCloneRef {
    PatternRoot {
        orig_index: Index<Pattern>,
        root_index: Index<Pattern>,
    },
    PatternChildOfPattern {
        index: Index<Pattern>,
        parent: Index<Chain<Pattern>>,
    },
    PatternChildOfTimedStep {
        index: Index<Pattern>,
        parent: Index<TimedStep>,
    },
    PatternChain {
        index: Index<Chain<Pattern>>,
        parent: Index<Pattern>,
    },
    TimedStep {
        index: Index<TimedStep>,
        parent: Index<Chain<TimedStep>>,
    },
    TimedStepChain {
        index: Index<Chain<TimedStep>>,
        parent: Index<Pattern>,
    },
}

pub enum PatternOrdRef {
    Pattern(Index<Pattern>),
    TimedStep(Index<TimedStep>),
}

/// Helper function to combine multiple iterators of different types.
pub fn combine_iters<T>(
    opt_iter1: Option<impl IntoIterator<Item = T>>,
    opt_iter2: Option<impl IntoIterator<Item = T>>,
) -> impl Iterator<Item = T> {
    opt_iter1
        .into_iter()
        .flatten()
        .chain(opt_iter2.into_iter().flatten())
}

fn get_multiple_refs<'a, Item: ArenaItem, T>(
    multiple: Multiple<Item>,
    arena: &'a impl Arena<Chain<Item>>,
    make_ref: impl Fn(Index<Chain<Item>>) -> T + 'a,
) -> impl Iterator<Item = ArenaResult<T>> + 'a {
    multiple
        .iter_with_chain(arena)
        .map(move |result| result.map(&make_ref))
}

pub fn pattern_refs<'a, T: 'a>(
    pattern: Pattern,
    arenas: &'a impl PatternArenas,
    from_pattern_chain_index: impl Fn(Index<Chain<Pattern>>) -> T + 'a,
    from_timed_step_chain_index: impl Fn(Index<Chain<TimedStep>>) -> T + 'a,
) -> impl Iterator<Item = ArenaResult<T>> + 'a {
    let (opt_iter1, opt_iter2) = match pattern {
        Pattern::Cat(multiple)
        | Pattern::Seq(multiple)
        | Pattern::Stack(multiple) => {
            let iter = get_multiple_refs(
                multiple,
                arenas.get_pattern_chain_arena(),
                from_pattern_chain_index,
            );
            (Some(iter), None)
        }
        Pattern::TimeCat(multiple) | Pattern::Arrange(multiple) => {
            let iter = get_multiple_refs(
                multiple,
                arenas.get_timed_step_chain_arena(),
                from_timed_step_chain_index,
            );
            (None, Some(iter))
        }
        Pattern::Note(_) | Pattern::Silence => (None, None),
    };

    combine_iters(opt_iter1, opt_iter2)
}
