use core::fmt::Debug;
use core::marker::PhantomData;

type IndexInner = u16;
const INVALID_INDEX_VALUE: u16 = u16::MAX;

/// An index type used to access an [crate::mem::Arena].
/// Because implicit copying can lead to hidden sharing of indices, which
/// violates the model of items in arenas having only one owner, this type is
/// not [Copy].
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Index<T>(IndexInner, PhantomData<T>);

impl<T: Debug> Debug for Index<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Index({})", self.0)
    }
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        self.const_clone()
    }
}

impl<T> Index<T> {
    #[inline(always)]
    pub const fn new(index: IndexInner) -> Self {
        Self(index, PhantomData)
    }

    pub const fn new_invalid() -> Self {
        Self::new(INVALID_INDEX_VALUE)
    }

    pub const fn const_clone(&self) -> Self {
        let Self(inner, _) = self;
        Index(*inner, PhantomData)
    }
}

impl<T> From<Index<T>> for usize {
    fn from(value: Index<T>) -> Self {
        let Index(inner, _) = value;
        usize::from(inner)
    }
}
