use super::index::Index;

#[allow(unused)]
#[derive(Debug)]
/// A type used to create a singly-linked arena-allocated list of items of type
/// `T`, where the `next` pointer is an [Index] to the same [Chain] type.
pub enum Chain<T> {
    Cons { head: Index<T>, tail: Index<Self> },
    Nil,
}
