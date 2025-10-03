use core::fmt::Debug;
use core::marker::PhantomData;

type IndexInner = u16;

/// An index type used to access an [super::Arena].
pub struct Index<T>(IndexInner, PhantomData<T>);

impl<T: Debug> Debug for Index<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Index({})", self.0)
    }
}

impl Index<()> {
    #[inline(always)]
    pub const fn transmute<T>(self) -> Index<T> {
        let Self(inner, _) = self;
        Index(inner, PhantomData)
    }
}

impl<T> Index<T> {
    #[inline(always)]
    pub const fn new(index: IndexInner) -> Self {
        Self(index, PhantomData)
    }

    #[inline(always)]
    pub const fn erase(self) -> Index<()> {
        let Self(inner, _) = self;
        Index(inner, PhantomData)
    }

    pub const fn copy(&self) -> Self {
        let Self(inner, _) = self;
        Self(*inner, PhantomData)
    }

    #[cfg(test)]
    pub const fn increment_by(&mut self, inc: u16) {
        self.0 += inc;
    }
}

impl<T> From<Index<T>> for usize {
    fn from(value: Index<T>) -> Self {
        let Index(inner, _) = value;
        usize::from(inner)
    }
}
