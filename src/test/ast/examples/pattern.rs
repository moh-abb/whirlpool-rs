use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::note::Note;
use crate::ast::note::NoteLetter;
use crate::ast::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Cow;
use crate::mem::FixableArena;
use crate::mem::Index;
use crate::mem::LinkedContainer;
use crate::mem::Multiple;
use crate::mem::SharedArena;
use crate::mem::linked::LinkedNode;

type FixedExprArenas = FixableArena<ExprNode>;

fn note_unit_node(letter: NoteLetter) -> Cow<ExprNode> {
    Cow::Owned(ExprNode::new_builtin_note_unit(NoteUnit::Note(Note::new(
        letter,
    ))))
}

fn silence_node() -> Cow<ExprNode> {
    Cow::Owned(ExprNode::new_builtin_silence())
}

fn timed_step_node(
    arenas: &mut impl ExprArenas,
    length: CycleTime,
) -> ArenaResult<Cow<ExprNode>> {
    let array_index = arenas.push(ExprNode(LinkedNode::new(Expr::Array {
        args: Multiple::new(),
    })))?;

    let shared_arena = SharedArena::new(arenas);
    let mut shared_arena_1 = shared_arena.make_ref();
    let mut shared_arena_2 = shared_arena.make_ref();

    let length_node =
        ExprNode(LinkedNode::new(Expr::Unit(ExprUnit::Number(length))));

    LinkedContainer::push_child(
        &mut shared_arena_1,
        &mut shared_arena_2,
        array_index.clone(),
        Cow::Owned(length_node),
    )?;

    Ok(Cow::Indexed(array_index))
}

fn one_cycle_with(node: ExprNode) -> (impl ExprArenas, Index<ExprNode>) {
    let head_index = Index::new(0);
    let arenas = FixedExprArenas::new([(head_index.clone(), node)]);
    (arenas, head_index)
}

pub fn one_cycle_unit(note: Note) -> (impl ExprArenas, Index<ExprNode>) {
    let node = ExprNode::new_builtin_note_unit(NoteUnit::Note(note));
    one_cycle_with(node)
}

pub fn one_cycle_silence() -> (impl ExprArenas, Index<ExprNode>) {
    one_cycle_with(ExprNode::new_builtin_silence())
}

pub fn binary_tree_depth_two(
    root: ExprNode,
    left_child: ExprNode,
    right_child: ExprNode,
) -> (impl ExprArenas, Index<ExprNode>) {
    // [ node ] -----v
    //    ↓          ↓
    // [ node ]  [ node ]
    //  ↓   ↓     ↓   ↓
    //  A   B     C   D
    let mut arenas = FixedExprArenas::new([]);
    arenas.allow_alloc();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

    let root_index = shared_arena_ref_1.push(root).unwrap();

    let left = LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_index.clone(),
        Cow::Owned(left_child),
    )
    .unwrap();

    let right = LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_index.clone(),
        Cow::Owned(right_child),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        left.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        left.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        right.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        right.clone(),
        note_unit_node(NoteLetter::D),
    )
    .unwrap();

    arenas.forbid_alloc();

    (arenas, root_index)
}

pub fn half_binary_tree_depth_two(
    root: ExprNode,
    child: ExprNode,
) -> (impl ExprArenas, Index<ExprNode>) {
    // [ root ] -----v
    //    ↓        ↓
    // [ child ]    C
    // ↓  ↓
    // A   B
    let mut arenas = FixedExprArenas::new([]);
    arenas.allow_alloc();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

    let root_tree = shared_arena_ref_1.push(root).unwrap();

    let left_tree = LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        Cow::Owned(child),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        left_tree.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();
    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        left_tree.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    arenas.forbid_alloc();

    (arenas, root_tree)
}

pub fn multiple_of_three_units(
    root: ExprNode,
) -> (impl ExprArenas, Index<ExprNode>) {
    // [ root ]
    // ↓ ↓ ↓
    // A  B  C
    let mut arenas = FixedExprArenas::new([]);
    arenas.allow_alloc();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

    let root_tree = shared_arena_ref_1.push(root).unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    arenas.forbid_alloc();

    (arenas, root_tree)
}

pub fn multiple_of_unit_then_silence_then_unit(
    root: ExprNode,
) -> (impl ExprArenas, Index<ExprNode>) {
    // [ root ]
    // ↓ ↓ ↓
    // A  ~  C
    let mut arenas = FixedExprArenas::new([]);
    arenas.allow_alloc();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

    let root_tree = shared_arena_ref_1.push(root).unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        silence_node(),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_tree.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    arenas.forbid_alloc();

    (arenas, root_tree)
}

pub fn multiple_of_four_timed_steps(
    root: ExprNode,
    elem_lengths: [CycleTime; 4],
) -> (impl ExprArenas, Index<ExprNode>) {
    // [  root  ]
    // ↓  ↓  ↓  ↓
    // A  B  C  D
    let letters = [NoteLetter::A, NoteLetter::B, NoteLetter::C, NoteLetter::D];

    let mut arenas = FixedExprArenas::new([]);
    arenas.allow_alloc();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

    let mut total_time = CycleTime::ZERO;
    for elem_length in elem_lengths {
        total_time = total_time.add(elem_length).unwrap();
    }

    let root_tree = shared_arena_ref_1.push(root).unwrap();

    for (elem_length, letter) in elem_lengths.into_iter().zip(letters) {
        let node =
            timed_step_node(&mut shared_arena_ref_1, elem_length).unwrap();

        let timed_step_index = LinkedContainer::push_child(
            &mut shared_arena_ref_1,
            &mut shared_arena_ref_2,
            root_tree.clone(),
            node,
        )
        .unwrap();

        // Add a single child to the timed step.
        LinkedContainer::push_child(
            &mut shared_arena_ref_1,
            &mut shared_arena_ref_2,
            timed_step_index,
            note_unit_node(letter),
        )
        .unwrap();
    }

    arenas.forbid_alloc();

    (arenas, root_tree)
}

pub fn half_binary_tree_depth_two_with_timed_steps(
    root: ExprNode,
    child: ExprNode,
    root_elem_lengths: [CycleTime; 2],
    child_elem_lengths: [CycleTime; 2],
) -> (impl ExprArenas, Index<ExprNode>) {
    // [  root  ] ---v
    //     |         |
    // root_elem_lengths
    //    ↓        ↓
    // [  child  ] C
    //  |   |
    // child_elem_lengths
    // ↓  ↓
    // A  B
    let mut arenas = FixedExprArenas::new([]);
    arenas.allow_alloc();
    let shared_arena = SharedArena::new(&mut arenas);
    let mut shared_arena_ref_1 = shared_arena.make_ref();
    let mut shared_arena_ref_2 = shared_arena.make_ref();

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

    let root_tree = shared_arena_ref_1.push(root).unwrap();

    let root_timed_steps = root_elem_lengths.map(|elem_length| {
        let timed_step =
            timed_step_node(&mut shared_arena_ref_1, elem_length).unwrap();

        LinkedContainer::push_child(
            &mut shared_arena_ref_1,
            &mut shared_arena_ref_2,
            root_tree.clone(),
            timed_step,
        )
        .unwrap()
    });

    let [root_timed_step_1, root_timed_step_2] = root_timed_steps;

    let child_tree = LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_timed_step_1,
        Cow::Owned(child),
    )
    .unwrap();

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        root_timed_step_2.clone(),
        note_unit_node(NoteLetter::C),
    )
    .unwrap();

    let child_timed_steps = child_elem_lengths.map(|elem_length| {
        let timed_step =
            timed_step_node(&mut shared_arena_ref_1, elem_length).unwrap();

        LinkedContainer::push_child(
            &mut shared_arena_ref_1,
            &mut shared_arena_ref_2,
            child_tree.clone(),
            timed_step,
        )
        .unwrap()
    });

    let [child_timed_step_1, child_timed_step_2] = child_timed_steps;

    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        child_timed_step_1.clone(),
        note_unit_node(NoteLetter::A),
    )
    .unwrap();
    LinkedContainer::push_child(
        &mut shared_arena_ref_1,
        &mut shared_arena_ref_2,
        child_timed_step_2.clone(),
        note_unit_node(NoteLetter::B),
    )
    .unwrap();

    arenas.forbid_alloc();

    (arenas, root_tree)
}
