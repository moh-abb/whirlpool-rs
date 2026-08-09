use crate::ast::CycleTime;
use crate::ast::NoteLetter;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::mem::Arena;
use crate::mem::Cow;
use crate::mem::FixableArena;
use crate::mem::Index;
use crate::mem::Multiple;

#[derive(Debug, Default)]
struct FixedArenas(FixableArena<PatternNode>);

impl PatternArenas for FixedArenas {
    fn get_pattern_arena(&self) -> &impl Arena<PatternNode> {
        &self.0
    }
}

fn note_unit_node(letter: NoteLetter) -> Cow<PatternNode> {
    Cow::Owned(PatternNode::new(Pattern::Note(NoteUnit::Letter(letter))))
}

fn silence_node() -> Cow<PatternNode> {
    Cow::Owned(PatternNode::new(Pattern::Silence))
}

fn timed_step_node(length: CycleTime) -> Cow<PatternNode> {
    Cow::Owned(PatternNode::new(Pattern::TimedStep(TimedStep(
        length,
        Multiple::new(),
    ))))
}

fn one_cycle_with(
    pattern: Pattern,
) -> (impl PatternArenas, Index<PatternNode>) {
    let head_index = Index::new(0);
    let arenas = FixedArenas(FixableArena::new([(
        head_index.clone(),
        PatternNode::new(pattern),
    )]));
    (arenas, head_index)
}

pub fn one_cycle_unit(
    unit: NoteUnit,
) -> (impl PatternArenas, Index<PatternNode>) {
    one_cycle_with(Pattern::Note(unit))
}

pub fn one_cycle_silence() -> (impl PatternArenas, Index<PatternNode>) {
    one_cycle_with(Pattern::Silence)
}

pub fn binary_tree_depth_two(
    root: fn(Multiple<PatternNode>) -> Pattern,
    left_child: fn(Multiple<PatternNode>) -> Pattern,
    right_child: fn(Multiple<PatternNode>) -> Pattern,
) -> (impl PatternArenas, Index<PatternNode>) {
    // [ node ] -----v
    //    ↓          ↓
    // [ node ]  [ node ]
    //  ↓   ↓     ↓   ↓
    //  A   B     C   D
    let mut arenas = FixedArenas(FixableArena::new([]));
    arenas.0.allow_alloc();

    let root_index = arenas
        .0
        .push(PatternNode::new(root(Multiple::new())))
        .unwrap();

    let left = Multiple::push_back(
        &arenas,
        root_index.clone(),
        Cow::Owned(PatternNode::new(left_child(Multiple::new()))),
    )
    .unwrap();

    let right = Multiple::push_back(
        &arenas,
        root_index.clone(),
        Cow::Owned(PatternNode::new(right_child(Multiple::new()))),
    )
    .unwrap();

    Multiple::push_back(&arenas, left.clone(), note_unit_node(NoteLetter::A))
        .unwrap();
    Multiple::push_back(&arenas, left.clone(), note_unit_node(NoteLetter::B))
        .unwrap();

    Multiple::push_back(&arenas, right.clone(), note_unit_node(NoteLetter::C))
        .unwrap();
    Multiple::push_back(&arenas, right.clone(), note_unit_node(NoteLetter::D))
        .unwrap();

    arenas.0.forbid_alloc();

    (arenas, root_index)
}

pub fn half_binary_tree_depth_two(
    root: fn(Multiple<PatternNode>) -> Pattern,
    child: fn(Multiple<PatternNode>) -> Pattern,
) -> (impl PatternArenas, Index<PatternNode>) {
    // [ root ] -----v
    //    ↓        ↓
    // [ child ]    C
    // ↓  ↓
    // A   B
    let mut arenas = FixedArenas(FixableArena::new([]));
    arenas.0.allow_alloc();

    let root_tree = arenas
        .0
        .push(PatternNode::new(root(Multiple::new())))
        .unwrap();

    let left_tree = Multiple::push_back(
        &arenas,
        root_tree.clone(),
        Cow::Owned(PatternNode::new(child(Multiple::new()))),
    )
    .unwrap();

    Multiple::push_back(
        &arenas,
        left_tree.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();
    Multiple::push_back(
        &arenas,
        left_tree.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();

    Multiple::push_back(
        &arenas,
        root_tree.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    arenas.0.forbid_alloc();

    (arenas, root_tree)
}

pub fn multiple_of_three_units(
    root: fn(Multiple<PatternNode>) -> Pattern,
) -> (impl PatternArenas, Index<PatternNode>) {
    // [ root ]
    // ↓ ↓ ↓
    // A  B  C
    let mut arenas = FixedArenas(FixableArena::new([]));
    arenas.0.allow_alloc();

    let root_tree = arenas
        .0
        .push(PatternNode::new(root(Multiple::new())))
        .unwrap();

    Multiple::push_back(
        &arenas,
        root_tree.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();
    Multiple::push_back(
        &arenas,
        root_tree.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();
    Multiple::push_back(
        &arenas,
        root_tree.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    arenas.0.forbid_alloc();

    (arenas, root_tree)
}

pub fn multiple_of_unit_then_silence_then_unit(
    root: fn(Multiple<PatternNode>) -> Pattern,
) -> (impl PatternArenas, Index<PatternNode>) {
    // [ root ]
    // ↓ ↓ ↓
    // A  ~  C
    let mut arenas = FixedArenas(FixableArena::new([]));
    arenas.0.allow_alloc();

    let root_tree = arenas
        .0
        .push(PatternNode::new(root(Multiple::new())))
        .unwrap();

    Multiple::push_back(
        &arenas,
        root_tree.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();
    Multiple::push_back(&arenas, root_tree.clone(), silence_node()).unwrap();
    Multiple::push_back(
        &arenas,
        root_tree.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    arenas.0.forbid_alloc();

    (arenas, root_tree)
}

pub fn multiple_of_four_timed_steps(
    root: fn(CycleTime, Multiple<PatternNode>) -> Pattern,
    elem_lengths: [CycleTime; 4],
) -> (impl PatternArenas, Index<PatternNode>) {
    // [  root  ]
    // ↓  ↓  ↓  ↓
    // A  B  C  D
    let letters = [NoteLetter::A, NoteLetter::B, NoteLetter::C, NoteLetter::D];

    let mut arenas = FixedArenas(FixableArena::new([]));
    arenas.0.allow_alloc();

    let mut total_time = CycleTime::ZERO;
    for elem_length in elem_lengths {
        total_time = total_time.add(elem_length).unwrap();
    }

    let root_tree = arenas
        .0
        .push(PatternNode::new(root(total_time, Multiple::new())))
        .unwrap();

    for (elem_length, letter) in elem_lengths.into_iter().zip(letters) {
        let timed_step_index = Multiple::push_back(
            &arenas,
            root_tree.clone(),
            timed_step_node(elem_length),
        )
        .unwrap();
        // Add a single child to the timed step.
        Multiple::push_back(&arenas, timed_step_index, note_unit_node(letter))
            .unwrap();
    }

    arenas.0.forbid_alloc();

    (arenas, root_tree)
}

pub fn half_binary_tree_depth_two_with_timed_steps(
    root: fn(CycleTime, Multiple<PatternNode>) -> Pattern,
    child: fn(CycleTime, Multiple<PatternNode>) -> Pattern,
    root_elem_lengths: [CycleTime; 2],
    child_elem_lengths: [CycleTime; 2],
) -> (impl PatternArenas, Index<PatternNode>) {
    // [  root  ] ---v
    //     |         |
    // root_elem_lengths
    //    ↓        ↓
    // [  child  ] C
    //  |   |
    // child_elem_lengths
    // ↓  ↓
    // A  B
    let mut arenas = FixedArenas(FixableArena::new([]));
    arenas.0.allow_alloc();

    let mut root_total_time = CycleTime::ZERO;
    for elem_length in root_elem_lengths {
        root_total_time = root_total_time
            .add(elem_length)
            .unwrap();
    }
    let mut child_total_time = CycleTime::ZERO;
    for elem_length in child_elem_lengths {
        child_total_time = child_total_time
            .add(elem_length)
            .unwrap();
    }

    let root_tree = arenas
        .0
        .push(PatternNode::new(root(root_total_time, Multiple::new())))
        .unwrap();

    let root_timed_steps = root_elem_lengths.map(|elem_length| {
        Multiple::push_back(
            &arenas,
            root_tree.clone(),
            timed_step_node(elem_length),
        )
        .unwrap()
    });

    let [root_timed_step_1, root_timed_step_2] = root_timed_steps;

    let child_tree = Multiple::push_back(
        &arenas,
        root_timed_step_1,
        Cow::Owned(PatternNode::new(child(child_total_time, Multiple::new()))),
    )
    .unwrap();

    Multiple::push_back(
        &arenas,
        root_timed_step_2.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    let child_timed_steps = child_elem_lengths.map(|elem_length| {
        Multiple::push_back(
            &arenas,
            child_tree.clone(),
            timed_step_node(elem_length),
        )
        .unwrap()
    });

    let [child_timed_step_1, child_timed_step_2] = child_timed_steps;

    Multiple::push_back(
        &arenas,
        child_timed_step_1.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();
    Multiple::push_back(
        &arenas,
        child_timed_step_2.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();

    arenas.0.forbid_alloc();

    (arenas, root_tree)
}
