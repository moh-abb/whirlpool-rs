use super::index::Index;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A type used to create a singly-linked arena-allocated list of items of type
/// `T`, where the `next` pointer is an [Index] to the same [Chain] type.
pub struct Chain<T>(
    pub Index<T>,
    pub Option<Index<Chain<T>>>,
    pub Option<Index<Chain<T>>>,
);

impl<T> Clone for Chain<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone())
    }
}
