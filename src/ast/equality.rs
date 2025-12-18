use core::mem::discriminant;

use crate::arena::Arena;
use crate::arena::chain::Chain;
use crate::arena::equality::ArenaEq;
use crate::arena::extension::Inspect;
use crate::arena::index::Index;
use crate::arena::tuple::DynArenasOf;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;

fn chain_iterators_eq<T: ArenaEq>(
    mut this_iter: impl Iterator<Item = T>,
    mut other_iter: impl Iterator<Item = T>,
    mut units_equal: impl FnMut(&T, &T) -> bool,
) -> bool {
    loop {
        match (this_iter.next(), other_iter.next()) {
            (None, None) => break true,
            (Some(this_value), Some(other_value))
                if units_equal(&this_value, &other_value) => {}
            _ => break false,
        }
    }
}

impl ArenaEq for TimedStep {
    fn eq_in<'a>(
        this: &'a Self,
        other: &'a Self,
        this_arenas: &DynArenasOf<'a, Self>,
        other_arenas: &DynArenasOf<'a, Self>,
    ) -> bool {
        let Self(this_unit, this_pattern_index) = this;
        let Self(other_unit, other_pattern_index) = other;
        if this_unit != other_unit {
            return false;
        }
        let (this_pattern_arena, _) = *this_arenas;
        let (other_pattern_arena, _) = *other_arenas;

        let clone_pat = |arena: &dyn Arena<Pattern>, index: &Index<Pattern>| {
            arena
                .inspect(index.clone(), Pattern::clone)
                .unwrap()
        };
        let this_pattern = clone_pat(this_pattern_arena, this_pattern_index);
        let other_pattern = clone_pat(other_pattern_arena, other_pattern_index);
        ArenaEq::eq_in(&this_pattern, &other_pattern, this_arenas, other_arenas)
    }
}

impl ArenaEq for Pattern {
    fn eq_in<'a>(
        this: &'a Self,
        other: &'a Self,
        this_arenas: &DynArenasOf<'a, Self>,
        other_arenas: &DynArenasOf<'a, Self>,
    ) -> bool {
        if discriminant(this) != discriminant(other) {
            return false;
        }

        let (
            this_pattern_arena,
            (
                this_chain_arena,
                (this_timed_step_arena, (this_timed_step_chain_arena, ())),
            ),
        ) = *this_arenas;
        let (
            other_pattern_arena,
            (
                other_chain_arena,
                (other_timed_step_arena, (other_timed_step_chain_arena, ())),
            ),
        ) = *other_arenas;

        let pattern_iter =
            |chain: &'a Chain<Pattern>, pattern_arena: _, chain_arena: _| {
                chain.iter(pattern_arena, chain_arena, Pattern::clone)
            };
        let timed_step_iter =
            |chain: &'a Chain<TimedStep>,
             timed_step_arena: _,
             timed_step_chain_arena: _| {
                chain.iter(
                    timed_step_arena,
                    timed_step_chain_arena,
                    TimedStep::clone,
                )
            };

        match (this, other) {
            (Self::Cat(this_chain), Self::Cat(other_chain))
            | (Self::Seq(this_chain), Self::Seq(other_chain))
            | (Self::Stack(this_chain), Self::Stack(other_chain)) => {
                chain_iterators_eq(
                    pattern_iter(
                        this_chain,
                        this_pattern_arena,
                        this_chain_arena,
                    ),
                    pattern_iter(
                        other_chain,
                        other_pattern_arena,
                        other_chain_arena,
                    ),
                    |this_unit, other_unit| {
                        ArenaEq::eq_in(
                            this_unit,
                            other_unit,
                            this_arenas,
                            other_arenas,
                        )
                    },
                )
            }
            (Self::TimeCat(this_chain), Self::TimeCat(other_chain)) => {
                chain_iterators_eq(
                    timed_step_iter(
                        this_chain,
                        this_timed_step_arena,
                        this_timed_step_chain_arena,
                    ),
                    timed_step_iter(
                        other_chain,
                        other_timed_step_arena,
                        other_timed_step_chain_arena,
                    ),
                    |this_unit, other_unit| {
                        ArenaEq::eq_in(
                            this_unit,
                            other_unit,
                            this_arenas,
                            other_arenas,
                        )
                    },
                )
            }
            (Self::Note(this_note), Self::Note(other_note)) => {
                this_note == other_note
            }
            (Self::Silence, Self::Silence) => true,
            _ => unreachable!(),
        }
    }
}
