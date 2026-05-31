use crate::mem::Index;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A type used to create a singly-linked arena-allocated list of items of type
/// `T`, where the `next` pointer is an [Index] to the same [Chain] type.
pub struct Chain<T>(Index<T>, Option<Index<Chain<T>>>, Option<Index<Chain<T>>>);

impl<T> Chain<T> {
    pub const fn new(index: Index<T>) -> Self {
        Self(index, None, None)
    }

    pub const fn new_invalid() -> Self {
        Self(Index::new_invalid(), None, None)
    }

    pub const fn get_index(&self) -> Index<T> {
        self.0.const_clone()
    }

    pub const fn set_index(&mut self, index: Index<T>) {
        self.0 = index;
    }

    pub const fn set_prev(&mut self, index: Index<Chain<T>>) {
        debug_assert!(self.1.is_none());
        self.1 = Some(index);
    }

    pub const fn set_next(&mut self, index: Index<Chain<T>>) {
        debug_assert!(self.2.is_none());
        self.2 = Some(index);
    }

    pub const fn pop_prev(&mut self) -> Option<Index<Chain<T>>> {
        self.1.take()
    }

    pub const fn pop_next(&mut self) -> Option<Index<Chain<T>>> {
        self.2.take()
    }
}

impl<T> Clone for Chain<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone())
    }
}
