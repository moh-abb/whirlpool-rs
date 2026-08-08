//! Used to represent types that can be borrowed either at zero cost
//! (for mutable references) or (for a [RefCell]) with a borrow guard.
//! To avoid panics caused by `RefCell`, borrows are fallible and enclosed with
//! an [ArenaResult].

use core::cell::RefCell;
use core::ops::DerefMut;

use crate::mem::ArenaError;
use crate::mem::ArenaResult;

mod private {
    pub trait Sealed {}
}

pub trait BorrowAdapter<T>: private::Sealed {
    fn try_borrow_mut(&mut self) -> ArenaResult<impl DerefMut<Target = T>>;
}

impl<T> private::Sealed for &RefCell<T> {}
impl<T> BorrowAdapter<T> for &RefCell<T> {
    fn try_borrow_mut(&mut self) -> ArenaResult<impl DerefMut<Target = T>> {
        RefCell::try_borrow_mut(self).map_err(|_| ArenaError::InvalidBorrow)
    }
}

impl<T> private::Sealed for &mut T {}
impl<T> BorrowAdapter<T> for &mut T {
    fn try_borrow_mut(&mut self) -> ArenaResult<impl DerefMut<Target = T>> {
        Ok(self.deref_mut())
    }
}
