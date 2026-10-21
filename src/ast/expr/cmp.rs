use core::cmp::Ordering;

use crate::ast::expr::ExprNode;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::ArenaError;
use crate::mem::Index;
use crate::mem::linked::cmp::cmp_linked;

#[derive(Debug)]
enum ArenaXorY<'a, ArenasX, ArenasY> {
    X(&'a ArenasX),
    Y(&'a ArenasY),
}

/// Used to compare two [Pattern]s, possibly from two different arenas.
#[derive(Debug)]
pub struct ExprOrdAdapter<'a, ArenasX, ArenasY> {
    index: Index<ExprNode>,
    arenas: ArenaXorY<'a, ArenasX, ArenasY>,
}

#[allow(unused)]
#[derive(derive_more::From, Debug)]
enum CmpError {
    ArenaErr(ArenaError),
    LeftRightMismatch,
}

impl<'a, ArenasX: ExprArenas, ArenasY: ExprArenas>
    ExprOrdAdapter<'a, ArenasX, ArenasY>
{
    #[allow(unused)]
    pub fn new_left(index: Index<ExprNode>, arenas: &'a ArenasX) -> Self {
        Self { index, arenas: ArenaXorY::X(arenas) }
    }

    #[allow(unused)]
    pub fn new_right(index: Index<ExprNode>, arenas: &'a ArenasY) -> Self {
        Self { index, arenas: ArenaXorY::Y(arenas) }
    }

    fn try_cmp(&self, other: &Self) -> Result<Ordering, CmpError> {
        match (&self.arenas, &other.arenas) {
            (&ArenaXorY::X(arenas_x), &ArenaXorY::Y(arenas_y)) => cmp_linked(
                self.index.clone(),
                other.index.clone(),
                arenas_x,
                arenas_y,
                Ord::cmp,
                ExprNode::clone,
            )
            .map_err(Into::into),
            _ => Err(CmpError::LeftRightMismatch),
        }
    }
}

/// Implementation of Ord for `Index<Pattern>`'s adapter
impl<'a, ArenasX: ExprArenas, ArenasY: ExprArenas> Ord
    for ExprOrdAdapter<'a, ArenasX, ArenasY>
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.try_cmp(other)
            .unwrap_or_else(|err| {
                panic!("Error encountered while comparing patterns: {err:?}")
            })
    }
}

impl<'a, Arenas1: ExprArenas, Arenas2: ExprArenas> PartialEq
    for ExprOrdAdapter<'a, Arenas1, Arenas2>
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<'a, Arenas1: ExprArenas, Arenas2: ExprArenas> Eq
    for ExprOrdAdapter<'a, Arenas1, Arenas2>
{
}

impl<'a, Arenas1: ExprArenas, Arenas2: ExprArenas> PartialOrd
    for ExprOrdAdapter<'a, Arenas1, Arenas2>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
