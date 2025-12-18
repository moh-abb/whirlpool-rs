use super::index::Index;

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A type used to create a singly-linked arena-allocated list of items of type
/// `T`, where the `next` pointer is an [Index] to the same [Chain] type.
pub enum Chain<T> {
    Cons { head: Index<T>, tail: Index<Self> },
    Nil,
}

impl<T> Clone for Chain<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Cons { head, tail } => {
                Self::Cons { head: head.clone(), tail: tail.clone() }
            }
            Self::Nil => Self::Nil,
        }
    }
}

#[derive(Debug)]
pub enum ChainOrIndex<T> {
    Chain(Chain<T>),
    Index(Index<Chain<T>>),
}
