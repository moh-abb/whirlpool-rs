use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::mem::Arena;
use crate::mem::arena::arena_impl::fixed_arena::FixedArena;
use crate::structures::chain::Chain;
use crate::structures::index::Index;
use crate::structures::multiple::Multiple;

#[derive(Debug, Default)]
struct FixedArenas {
    pattern_arena: FixedArena<Pattern>,
    pattern_chain_arena: FixedArena<Chain<Pattern>>,
    timed_step_arena: FixedArena<TimedStep>,
    timed_step_chain_arena: FixedArena<Chain<TimedStep>>,
}

impl PatternArenas for FixedArenas {
    fn get_pattern_arena(&self) -> &impl Arena<Pattern> {
        &self.pattern_arena
    }

    fn get_pattern_chain_arena(&self) -> &impl Arena<Chain<Pattern>> {
        &self.pattern_chain_arena
    }

    fn get_timed_step_arena(&self) -> &impl Arena<TimedStep> {
        &self.timed_step_arena
    }

    fn get_timed_step_chain_arena(&self) -> &impl Arena<Chain<TimedStep>> {
        &self.timed_step_chain_arena
    }
}

fn one_cycle_with(pattern: Pattern) -> (impl PatternArenas, Index<Pattern>) {
    let head_index = Index::new(0);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([(head_index.clone(), pattern)]),
        ..Default::default()
    };
    (arenas, head_index)
}

pub fn one_cycle_unit(unit: NoteUnit) -> (impl PatternArenas, Index<Pattern>) {
    one_cycle_with(Pattern::Note(unit))
}

pub fn one_cycle_silence() -> (impl PatternArenas, Index<Pattern>) {
    one_cycle_with(Pattern::Silence)
}

pub fn binary_tree_depth_two(
    root: fn(Multiple<Pattern>) -> Pattern,
    left_child: fn(Multiple<Pattern>) -> Pattern,
    right_child: fn(Multiple<Pattern>) -> Pattern,
) -> (impl PatternArenas, Index<Pattern>) {
    // [ node ] -----v
    //    ↓        ↓
    // [ node ]  [ node ]
    // ↓  ↓    ↓  ↓
    // A   B     C   D
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let left_child_multiple =
        Multiple::new_nonempty(2, Index::new(0xC0), Index::new(0xC1));
    let right_child_multiple =
        Multiple::new_nonempty(2, Index::new(0xC2), Index::new(0xC3));
    let root_multiple =
        Multiple::new_nonempty(2, Index::new(0xC4), Index::new(0xC5));
    let head_index = Index::new(6);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(2), Pattern::Note(note_unit(Letter::C))),
            (Index::new(3), Pattern::Note(note_unit(Letter::D))),
            (Index::new(4), left_child(left_child_multiple)),
            (Index::new(5), right_child(right_child_multiple)),
            (head_index.clone(), root(root_multiple)),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(Index::new(1), Some(Index::new(0xC0)), None),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(2), None, Some(Index::new(0xC3))),
            ),
            (
                Index::new(0xC3),
                Chain(Index::new(3), Some(Index::new(0xC2)), None),
            ),
            (
                Index::new(0xC4),
                Chain(Index::new(4), None, Some(Index::new(0xC5))),
            ),
            (
                Index::new(0xC5),
                Chain(Index::new(5), Some(Index::new(0xC4)), None),
            ),
        ]),
        ..Default::default()
    };
    (arenas, head_index)
}

pub fn half_binary_tree_depth_two(
    root: fn(Multiple<Pattern>) -> Pattern,
    child: fn(Multiple<Pattern>) -> Pattern,
) -> (impl PatternArenas, Index<Pattern>) {
    // [ root ] -----v
    //    ↓        ↓
    // [ child ]    C
    // ↓  ↓
    // A   B
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let left_tree =
        child(Multiple::new_nonempty(2, Index::new(0xC0), Index::new(0xC1)));
    let tree =
        root(Multiple::new_nonempty(2, Index::new(0xC2), Index::new(0xC3)));
    let head_index = Index::new(6);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(4), left_tree),
            (Index::new(5), Pattern::Note(note_unit(Letter::C))),
            (head_index.clone(), tree),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(Index::new(1), Some(Index::new(0xC0)), None),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(4), None, Some(Index::new(0xC3))),
            ),
            (
                Index::new(0xC3),
                Chain(Index::new(5), Some(Index::new(0xC2)), None),
            ),
        ]),
        ..Default::default()
    };
    (arenas, head_index)
}

pub fn multiple_of_three_units(
    root: fn(Multiple<Pattern>) -> Pattern,
) -> (impl PatternArenas, Index<Pattern>) {
    // [ root ]
    // ↓ ↓ ↓
    // A  B  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let overall_pattern =
        root(Multiple::new_nonempty(3, Index::new(0xC0), Index::new(0xC2)));
    let head_index = Index::new(3);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(2), Pattern::Note(note_unit(Letter::C))),
            (head_index.clone(), overall_pattern),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(
                    Index::new(1),
                    Some(Index::new(0xC0)),
                    Some(Index::new(0xC2)),
                ),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(2), Some(Index::new(0xC1)), None),
            ),
        ]),
        ..Default::default()
    };
    (arenas, head_index)
}

pub fn multiple_of_unit_then_silence_then_unit(
    root: fn(Multiple<Pattern>) -> Pattern,
) -> (impl PatternArenas, Index<Pattern>) {
    // [ root ]
    // ↓ ↓ ↓
    // A  ~  C
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let overall_pattern =
        root(Multiple::new_nonempty(3, Index::new(0xC0), Index::new(0xC2)));
    let head_index = Index::new(3);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Silence),
            (Index::new(2), Pattern::Note(note_unit(Letter::C))),
            (head_index.clone(), overall_pattern),
        ]),
        pattern_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(
                    Index::new(1),
                    Some(Index::new(0xC0)),
                    Some(Index::new(0xC2)),
                ),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(2), Some(Index::new(0xC1)), None),
            ),
        ]),
        ..Default::default()
    };
    (arenas, head_index)
}

pub fn multiple_of_four_timed_steps(
    root: fn(Multiple<TimedStep>) -> Pattern,
    elem_lengths: [CycleTime; 4],
) -> (impl PatternArenas, Index<Pattern>) {
    // [  root  ]
    // ↓  ↓  ↓  ↓
    // A  B  C  D
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let overall_pattern =
        root(Multiple::new_nonempty(4, Index::new(0xD0), Index::new(0xD3)));
    let head_index = Index::new(4);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(2), Pattern::Note(note_unit(Letter::C))),
            (Index::new(3), Pattern::Note(note_unit(Letter::D))),
            (head_index.clone(), overall_pattern),
        ]),
        timed_step_arena: FixedArena::new([
            (Index::new(0xA0), TimedStep(elem_lengths[0], Index::new(0))),
            (Index::new(0xA1), TimedStep(elem_lengths[1], Index::new(1))),
            (Index::new(0xA2), TimedStep(elem_lengths[2], Index::new(2))),
            (Index::new(0xA3), TimedStep(elem_lengths[3], Index::new(3))),
        ]),
        timed_step_chain_arena: FixedArena::new([
            (
                Index::new(0xD0),
                Chain(Index::new(0xA0), None, Some(Index::new(0xD1))),
            ),
            (
                Index::new(0xD1),
                Chain(
                    Index::new(0xA1),
                    Some(Index::new(0xD0)),
                    Some(Index::new(0xD2)),
                ),
            ),
            (
                Index::new(0xD2),
                Chain(
                    Index::new(0xA2),
                    Some(Index::new(0xD1)),
                    Some(Index::new(0xD3)),
                ),
            ),
            (
                Index::new(0xD3),
                Chain(Index::new(0xA3), Some(Index::new(0xD2)), None),
            ),
        ]),
        ..Default::default()
    };
    (arenas, head_index)
}

pub fn half_binary_tree_depth_two_with_timed_steps(
    root: fn(Multiple<TimedStep>) -> Pattern,
    child: fn(Multiple<TimedStep>) -> Pattern,
    root_elem_lengths: [CycleTime; 2],
    child_elem_lengths: [CycleTime; 2],
) -> (impl PatternArenas, Index<Pattern>) {
    // [ root@6 ] ---v
    //     |         |
    // root_elem_lengths
    //    ↓        ↓
    // [ child@4 ]  C@5
    //  |   |
    // child_elem_lengths
    // ↓  ↓
    // A@0 B@1
    let note_unit = |letter: Letter| NoteUnit::Letter(letter);
    let left_tree =
        child(Multiple::new_nonempty(2, Index::new(0xC2), Index::new(0xC3)));
    let tree =
        root(Multiple::new_nonempty(2, Index::new(0xC0), Index::new(0xC1)));
    let head_index = Index::new(6);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([
            (Index::new(0), Pattern::Note(note_unit(Letter::A))),
            (Index::new(1), Pattern::Note(note_unit(Letter::B))),
            (Index::new(4), left_tree),
            (Index::new(5), Pattern::Note(note_unit(Letter::C))),
            (head_index.clone(), tree),
        ]),
        pattern_chain_arena: FixedArena::default(),
        timed_step_arena: FixedArena::new([
            (Index::new(0xD0), TimedStep(root_elem_lengths[0], Index::new(4))),
            (Index::new(0xD1), TimedStep(root_elem_lengths[1], Index::new(5))),
            (Index::new(0xD2), TimedStep(child_elem_lengths[0], Index::new(0))),
            (Index::new(0xD3), TimedStep(child_elem_lengths[1], Index::new(1))),
        ]),
        timed_step_chain_arena: FixedArena::new([
            (
                Index::new(0xC0),
                Chain(Index::new(0xD0), None, Some(Index::new(0xC1))),
            ),
            (
                Index::new(0xC1),
                Chain(Index::new(0xD1), Some(Index::new(0xC0)), None),
            ),
            (
                Index::new(0xC2),
                Chain(Index::new(0xD2), None, Some(Index::new(0xC3))),
            ),
            (
                Index::new(0xC3),
                Chain(Index::new(0xD3), Some(Index::new(0xC2)), None),
            ),
        ]),
    };
    (arenas, head_index)
}
