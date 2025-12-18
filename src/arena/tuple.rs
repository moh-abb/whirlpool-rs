//! Definitions for right-associated tuples, and implementing traits for them.

use super::Arena;
use super::ArenaItem;
use super::handler::ArenaHandler;

mod right_tuple {
    pub trait Sealed {}
    impl Sealed for () {}
    impl<T, TS: super::RightTuple> Sealed for (T, TS) {}
}

mod arena_tuple {
    pub trait Sealed {}
    impl Sealed for &() {}
    impl<A, AS> Sealed for &(A, AS) {}
}

/// A trait which is satisfied by the types:
/// - `()` and `&()` (base cases)
/// - `(_, ())` and `(_, &())`
/// - `(_, (_, ()))` and `(_, (_, &()))`
/// - and so on (recursive cases).
pub trait RightTuple: right_tuple::Sealed {}
impl<T: right_tuple::Sealed> RightTuple for T {}

/// A [RightTuple] of references to (statically dispatched) [Arena]s.
/// The `T` refers to these references.
/// Encompasses the types:
/// - `&()` (base case)
/// - `(&'a impl Arena<_>, &'a ())`
/// - `(&'a impl Arena<_>, (&'a impl Arena<_>, &'a ()))`
/// - and so on (the lifetimes are equal).
///
/// Provides an associated type where all the [Arena] references are
/// dynamically dispatched.
pub trait ArenaTuple<T: RightTuple>: arena_tuple::Sealed + Copy {
    /// A [RightTuple] of references to dynamically-dispatched [Arena]s.
    /// Encompasses the types:
    /// - `()` (base case)
    /// - `(&'a dyn Arena<_>, ())`
    /// - `(&'a dyn Arena<_>, (&'a dyn Arena<_>, ()))`
    /// - and so on.
    type DynArenaTuple: RightTuple;
    /// Converts this tuple of [Arena]s to its dynamically-dispatched
    /// equivalent.
    fn to_dyn_arenas(self) -> Self::DynArenaTuple;
}

impl ArenaTuple<()> for &'_ () {
    type DynArenaTuple = ();
    fn to_dyn_arenas(self) -> Self::DynArenaTuple {
        *self
    }
}

impl<'a, T, A, TS, AS> ArenaTuple<(T, TS)> for &'a (A, AS)
where
    T: ArenaItem,
    A: Arena<T>,
    TS: RightTuple,
    &'a AS: ArenaTuple<TS>,
{
    type DynArenaTuple =
        (&'a dyn Arena<T>, <&'a AS as ArenaTuple<TS>>::DynArenaTuple);

    fn to_dyn_arenas(self) -> Self::DynArenaTuple {
        let (head, tail) = self;
        (head as &dyn Arena<T>, ArenaTuple::<TS>::to_dyn_arenas(tail))
    }
}

/// Shorthand for an [ArenaTuple] of a given [ArenaHandler] `H`, for which
/// `H`'s indices are exactly equal to this item's.
pub trait ArenaHandlerTuple<'a, H>
where
    H: ArenaHandler + ?Sized,
    Self: ArenaTuple<H::Indices, DynArenaTuple = H::DynArenas<'a>>,
{
}
impl<'a, H, AT> ArenaHandlerTuple<'a, H> for AT
where
    H: ArenaHandler + ?Sized,
    AT: ArenaTuple<H::Indices, DynArenaTuple = H::DynArenas<'a>>,
{
}

/// Shorthand for the associated type of an [ArenaHandler]'s [DynArenas].
#[allow(unused)]
pub type DynArenasOf<'a, T> = <T as ArenaHandler>::DynArenas<'a>;
