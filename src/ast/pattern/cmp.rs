use core::cmp::Ordering;

use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::pattern_discriminant;
use crate::mem::Index;
use crate::mem::linked::cmp_linked;

/// Compares two [PatternNode]s on a surface level, without considering nesting
/// between the nodes' children.
fn cmp_pattern_nodes(node_x: &PatternNode, node_y: &PatternNode) -> Ordering {
    let pattern_x = &node_x.pattern;
    let pattern_y = &node_y.pattern;
    let cmp_discriminant =
        pattern_discriminant(pattern_x).cmp(&pattern_discriminant(pattern_y));
    // We only need to handle the cases where there is extra data attached
    // to the Pattern separately.
    match (pattern_x, pattern_y) {
        (
            Pattern::TimedStep(TimedStep(time_x, _)),
            Pattern::TimedStep(TimedStep(time_y, _)),
        ) => {
            // Comparison will continue with the singular child
            time_x.cmp(time_y)
        }
        (Pattern::Note(note_x), Pattern::Note(note_y)) => note_x.cmp(note_y),
        _ => {
            // Comparison will continue with the chidren (or none for Silence)
            cmp_discriminant
        }
    }
}

#[derive(Debug)]
enum ArenaXorY<'a, ArenasX, ArenasY> {
    X(&'a ArenasX),
    Y(&'a ArenasY),
}

/// Used to compare two [Pattern]s, possibly from two different arenas.
#[derive(Debug)]
pub struct PatternOrdAdapter<'a, ArenasX, ArenasY> {
    index: Index<PatternNode>,
    arenas: ArenaXorY<'a, ArenasX, ArenasY>,
}

impl<'a, ArenasX: PatternArenas, ArenasY: PatternArenas>
    PatternOrdAdapter<'a, ArenasX, ArenasY>
{
    #[allow(unused)]
    pub fn new_left(index: Index<PatternNode>, arenas: &'a ArenasX) -> Self {
        Self { index, arenas: ArenaXorY::X(arenas) }
    }

    #[allow(unused)]
    pub fn new_right(index: Index<PatternNode>, arenas: &'a ArenasY) -> Self {
        Self { index, arenas: ArenaXorY::Y(arenas) }
    }
}

/// Implementation of Ord for `Index<Pattern>`'s adapter
impl<'a, ArenasX: PatternArenas, ArenasY: PatternArenas> Ord
    for PatternOrdAdapter<'a, ArenasX, ArenasY>
{
    fn cmp(&self, other: &Self) -> Ordering {
        let opt_comparison_result = match (&self.arenas, &other.arenas) {
            (&ArenaXorY::X(arenas_x), &ArenaXorY::Y(arenas_y)) => cmp_linked(
                self.index.clone(),
                other.index.clone(),
                arenas_x,
                arenas_y,
                cmp_pattern_nodes,
                PatternNode::clone,
            ),
            _ => panic!(
                "Need to use new_left for left hand side \
                and new_right for right hand side"
            ),
        };

        let cmp_result = opt_comparison_result.unwrap_or_else(|err| {
            panic!("Error encountered while comparing patterns: {err:?}")
        });
        cmp_result
    }
}

/// Default implementation of PartialEq for `Index<Pattern>`'s adapter
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PartialEq
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

/// Default implementation of [Eq] for `Index<Pattern>`'s adapter.
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> Eq
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
}

/// Default implementation of [PartialOrd] for `Index<Pattern>`'s adapter.
impl<'a, Arenas1: PatternArenas, Arenas2: PatternArenas> PartialOrd
    for PatternOrdAdapter<'a, Arenas1, Arenas2>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
