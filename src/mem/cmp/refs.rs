use core::cmp::Ordering;
use core::ops::ControlFlow;

use crate::mem::ArenaResult;

pub trait NextOrdRefs<'a, AX, AY, T: OrdRefs<AX, AY>>
where
    Self: Iterator<
            Item = (ArenaResult<T::OrdRefType>, ArenaResult<T::OrdRefType>),
        > + 'a,
{
}
impl<'a, I, AX, AY, T: OrdRefs<AX, AY>> NextOrdRefs<'a, AX, AY, T> for I where
    Self: Iterator<
            Item = (ArenaResult<T::OrdRefType>, ArenaResult<T::OrdRefType>),
        > + 'a
{
}

pub trait OrdRefs<ArenasX, ArenasY>: Sized {
    type OrdRefType;

    fn start_refs(
        start_x: Self,
        start_y: Self,
    ) -> (Self::OrdRefType, Self::OrdRefType);

    fn process_ref<'a>(
        reference_x: Self::OrdRefType,
        reference_y: Self::OrdRefType,
        arenas_x: &'a ArenasX,
        arenas_y: &'a ArenasY,
    ) -> ArenaResult<
        ControlFlow<Ordering, impl NextOrdRefs<'a, ArenasX, ArenasY, Self>>,
    >;
}
