//! Used to represent types that can be borrowed either at zero cost
//! (for mutable references) or (for a [RefCell]) with a borrow guard.

use core::cell::RefCell;
use core::ops::DerefMut;

mod private {
    pub trait Sealed {}
}

pub trait BorrowAdapter<T>: private::Sealed {
    fn borrow_mut(&mut self) -> impl DerefMut<Target = T>;
}

impl<T> private::Sealed for &RefCell<T> {}
impl<T> BorrowAdapter<T> for &RefCell<T> {
    fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
        RefCell::borrow_mut(self)
    }
}

impl<T> private::Sealed for &mut T {}
impl<T> BorrowAdapter<T> for &mut T {
    fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
        self.deref_mut()
    }
}
