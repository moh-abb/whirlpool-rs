use core::marker::PhantomData;
use core::num::NonZeroU16;

use derive_where::derive_where;

type IndexInput = u16;
type IndexInner = NonZeroU16;
const INVALID_INDEX_VALUE: u16 = u16::MAX;

/// An index type used to access an [crate::mem::Arena].
/// Because implicit copying can lead to hidden sharing of indices, which
/// violates the model of items in arenas having only one owner, this type is
/// not [Copy].
#[derive(derive_more::Debug)]
#[derive_where(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index<T>(IndexInner, PhantomData<T>);

impl<T> Index<T> {
    #[inline(always)]
    pub const fn new(index: IndexInput) -> Self {
        const ONE: NonZeroU16 = NonZeroU16::new(1).unwrap();
        Self(ONE.saturating_add(index), PhantomData)
    }

    pub const fn new_invalid() -> Self {
        Self::new(INVALID_INDEX_VALUE)
    }
}

impl<T> From<Index<T>> for usize {
    fn from(value: Index<T>) -> Self {
        let Index(inner, _) = value;
        // Scale the index down by one for zero-indexed values.
        let scaled_down = inner.get().saturating_sub(1);
        usize::from(scaled_down)
    }
}
