use crate::mem::Index;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A type used to create a singly-linked arena-allocated list of items of type
/// `T`, where the `next` pointer is an [Index] to the same [Chain] type.
pub struct Chain<T> {
    pub(super) prev: Option<Index<T>>,
    pub(super) next: Option<Index<T>>,
}

impl<T> Chain<T> {
    pub const fn new() -> Self {
        Self { prev: None, next: None }
    }

    pub fn prev(&self) -> Option<Index<T>> {
        self.prev.clone()
    }

    pub fn next(&self) -> Option<Index<T>> {
        self.next.clone()
    }
}

impl<T> Clone for Chain<T> {
    fn clone(&self) -> Self {
        Self { prev: self.prev.clone(), next: self.next.clone() }
    }
}
