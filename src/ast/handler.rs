use super::pattern::Pattern;
use super::pattern::TimedStep;
use crate::arena::Arena;
use crate::arena::chain::Chain;
use crate::arena::chain::ChainOrIndex;
use crate::arena::extension::Inspect;
use crate::arena::handler::ArenaHandler;
use crate::arena::tuple::ArenaHandlerTuple;
use crate::handle_dyn_arenas;
use crate::handle_indices;

/// Helper function to drop a [Chain].
fn chain_drop<'a, T: ArenaHandler>(
    chain: Chain<T>,
    main_arena: &dyn Arena<T>,
    chain_arena: &dyn Arena<Chain<T>>,
    item_arenas: &'a impl ArenaHandlerTuple<'a, T>,
) {
    let mut acc = ChainOrIndex::Chain(chain);
    loop {
        match acc {
            ChainOrIndex::Chain(Chain::Nil) => break,
            ChainOrIndex::Chain(Chain::Cons { head, tail }) => {
                let head_value = main_arena
                    .take(head)
                    .expect("[chain_drop]: main arena should have had head");
                head_value.drop_in(item_arenas);
                acc = ChainOrIndex::Index(tail);
            }
            ChainOrIndex::Index(index) => {
                let subchain = chain_arena
                    .take(index)
                    .expect("[chain_drop]: chain arena should have had index");
                acc = ChainOrIndex::Chain(subchain);
            }
        }
    }
}

/// Helper function to clone a [Chain].
#[allow(unused)]
fn chain_clone<'a, T: ArenaHandler>(
    chain: &Chain<T>,
    main_arena: &dyn Arena<T>,
    chain_arena: &dyn Arena<Chain<T>>,
    item_dyn_arenas: &'a impl ArenaHandlerTuple<'a, T>,
) -> Chain<T> {
    todo!()
}

impl ArenaHandler for TimedStep {
    type Indices =
        handle_indices!(Pattern, Chain<Pattern>, TimedStep, Chain<TimedStep>);

    type DynArenas<'a> = handle_dyn_arenas!('a, Pattern, Chain<Pattern>, TimedStep, Chain<TimedStep>);

    fn drop_in<'a>(self, arenas: &'a impl ArenaHandlerTuple<'a, Self>) {
        let (
            pattern_arena,
            (_chain_arena, (_timed_step_arena, (_timed_step_chain_arena, ()))),
        ) = arenas.to_dyn_arenas();
        let Self(_time_unit, pattern_index) = self;
        pattern_arena
            .take(pattern_index.clone())
            .unwrap()
            .drop_in(arenas);
    }

    fn clone_in<'a>(
        &self,
        arenas: &'a impl ArenaHandlerTuple<'a, Self>,
    ) -> Self {
        let (
            pattern_arena,
            (_chain_arena, (_timed_step_arena, (_timed_step_chain_arena, ()))),
        ) = arenas.to_dyn_arenas();
        let Self(_time_unit, pattern_index) = self;
        let cloned_pattern = pattern_arena
            .inspect(pattern_index.clone(), |pattern| pattern.clone_in(arenas))
            .unwrap();
        let cloned_pattern_index = pattern_arena
            .alloc(cloned_pattern)
            .unwrap();
        Self(*_time_unit, cloned_pattern_index)
    }
}

impl ArenaHandler for Pattern {
    type Indices =
        handle_indices!(Pattern, Chain<Pattern>, TimedStep, Chain<TimedStep>);

    type DynArenas<'a> = handle_dyn_arenas!('a, Pattern, Chain<Pattern>, TimedStep, Chain<TimedStep>);

    fn clone_in<'a>(
        &self,
        arenas: &'a impl ArenaHandlerTuple<'a, Self>,
    ) -> Self {
        let (
            pattern_arena,
            (chain_arena, (timed_step_arena, (timed_step_chain_arena, ()))),
        ) = arenas.to_dyn_arenas();

        match self {
            Self::Cat(chain) | Self::Seq(chain) | Self::Stack(chain) => {
                let constructor = match self {
                    Self::Cat(_) => Self::Cat,
                    Self::Seq(_) => Self::Seq,
                    Self::Stack(_) => Self::Stack,
                    _ => unreachable!(),
                };
                constructor(chain_clone(
                    chain,
                    pattern_arena,
                    chain_arena,
                    arenas,
                ))
            }
            Self::TimeCat(timed_steps) => Self::TimeCat(chain_clone(
                timed_steps,
                timed_step_arena,
                timed_step_chain_arena,
                arenas,
            )),
            Self::Note(n) => Self::Note(*n),
            Self::Silence => Self::Silence,
        }
    }

    fn drop_in<'a>(self, arenas: &'a impl ArenaHandlerTuple<'a, Self>) {
        let (
            pattern_arena,
            (chain_arena, (timed_step_arena, (timed_step_chain_arena, ()))),
        ) = arenas.to_dyn_arenas();

        match self {
            Self::Cat(chain) | Self::Seq(chain) | Self::Stack(chain) => {
                chain_drop(chain, pattern_arena, chain_arena, arenas);
            }
            Self::TimeCat(timed_steps) => chain_drop(
                timed_steps,
                timed_step_arena,
                timed_step_chain_arena,
                arenas,
            ),
            Self::Note(_n) => (),
            Self::Silence => (),
        }
    }
}
