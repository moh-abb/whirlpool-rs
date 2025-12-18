use super::ArenaItem;
use super::tuple::RightTuple;
use crate::arena::tuple::DynArenasOf;

/// A trait to specify how the given arena item should be dropped and cloned.
/// We only implement cloning and dropping for an [ArenaItem]
/// confined to the same [super::Arena] in which it was created. It is
/// a logic error to clone or drop this item by passing arenas which are
/// different to those it was created in, because if the item points to a given
/// [super::Index], then this will be treated as a dangling index.
#[allow(dead_code)]
pub trait ArenaHandler: ArenaItem {
    /// A right-associated tuple of types for which this type may have arenas,
    /// where the rightmost element is `()`.
    ///
    /// For example, if `enum Expr` had variants `Add(Index<Expr>, Index<Expr>)`
    /// and `Unit(Index<Atom>)`, then the type would be
    /// `(Expr, (Atom, ()))`
    type Indices: RightTuple;
    /// A right-associated tuple of `&'a dyn Arena<A>` of the same length as
    /// [ArenaHandler::Indices], where `A` is the corresponding type in
    /// [ArenaHandler::Indices], and the rightmost element is `()` (not a
    /// reference).
    ///
    /// For example, if `enum Expr` had variants `Add(Index<Expr>, Index<Expr>)`
    /// and `Unit(Index<Atom>)`, then the value would be
    /// `(&'a dyn Arena<Expr>, (&'a dyn Arena<Atom>, ()))`.
    type DynArenas<'a>: RightTuple;

    /// Drops this item from the given `arenas`, assuming that it was
    /// created in these `arenas`.
    fn drop_in<'a>(self, arenas: &DynArenasOf<'a, Self>);

    /// Clones this item using space in the given `arenas`, assuming that it
    /// was created in these `arenas`.
    fn clone_in<'a>(&self, arenas: &DynArenasOf<'a, Self>) -> Self;
}
