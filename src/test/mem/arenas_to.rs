use core::fmt::Debug;

use crate::mem::Rc;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;

/// Traits representing functions with static lifetimes, that take a mutable
/// reference to `Arenas` and produce an [ArenaResult].
/// This can be thought of as an impure generator function which can modify the
/// arena.
pub trait ArenasToFn<'r, Arenas, T, E>
where
    Self: Fn(SharedArenaRef<'r, Arenas>) -> Result<T, E> + 'r,
    Arenas: 'r,
{
}
impl<'r, Arenas, T, E, F> ArenasToFn<'r, Arenas, T, E> for F
where
    Self: Fn(SharedArenaRef<'r, Arenas>) -> Result<T, E> + 'r,
    Arenas: 'r,
{
}

/// A helper struct to avoid rewriting casting to [ArenasToFn].
pub struct ArenasTo<'r, Arenas, T, E>(
    Rc<dyn ArenasToFn<'r, Arenas, T, E> + 'r>,
);

impl<'r, Arenas, T, E> Debug for ArenasTo<'r, Arenas, T, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ArenasTo")
    }
}

impl<'r, Arenas, T, E> Clone for ArenasTo<'r, Arenas, T, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'r, Arenas, T, E> ArenasTo<'r, Arenas, T, E> {
    pub fn new(f: impl ArenasToFn<'r, Arenas, T, E>) -> Self {
        Self(Rc::new(f) as Rc<dyn ArenasToFn<'r, Arenas, T, E>>)
    }

    pub fn call(&self, arenas: SharedArenaRef<'r, Arenas>) -> Result<T, E> {
        (self.0)(arenas)
    }
}
