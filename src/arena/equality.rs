use super::handler::ArenaHandler;
use super::tuple::DynArenasOf;

/// A trait to define equality on arena-allocated types.
#[allow(unused)]
pub trait ArenaEq: ArenaHandler {
    /// Returns true if and only if `this`, assuming its indices
    /// ([super::Index]) point to values in `this_arenas`,
    /// is taken to be equal to `other`, whose indices point to values in
    /// `other_arenas`.
    ///
    /// The indices in `this` are taken point to those in `this_arenas` (and
    /// the same for `other` and `other_arenas`), but we cannot assume that
    /// simply because `this` and `other` refer to an equal index, that this
    /// means the values at that index in their corresponding arenas are equal.
    fn eq_in<'a>(
        this: &Self,
        other: &Self,
        this_arenas: &DynArenasOf<'a, Self>,
        other_arenas: &DynArenasOf<'a, Self>,
    ) -> bool;
}
