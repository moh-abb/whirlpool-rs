use core::fmt::Debug;

use crate::alloc_types::Rc;
use crate::arena::error::ArenaResult;
use crate::ast::pattern::arenas::PatternArenas;

/// Traits representing functions with static lifetimes, that take a tuple of
/// `dyn Arena<_>` and produce an [ArenaResult].
/// This can be thought of as an impure generator function which can modify the
/// arena.
pub trait ArenasToFn<Arenas, T>
where
    Self: Fn(&Arenas) -> ArenaResult<T>,
    Arenas: PatternArenas,
{
}
impl<Arenas, T, F> ArenasToFn<Arenas, T> for F
where
    Self: Fn(&Arenas) -> ArenaResult<T>,
    Arenas: PatternArenas,
{
}

/// A helper struct to avoid rewriting casting to [ArenasToFn].
pub struct ArenasTo<Arenas, T>(Rc<dyn ArenasToFn<Arenas, T>>);

impl<Arenas, T> Debug for ArenasTo<Arenas, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ArenasTo")
    }
}

impl<Arenas, T> Clone for ArenasTo<Arenas, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Arenas: PatternArenas, T> ArenasTo<Arenas, T> {
    pub fn new(f: impl ArenasToFn<Arenas, T> + 'static) -> Self {
        Self(Rc::new(f) as Rc<dyn ArenasToFn<Arenas, T>>)
    }

    pub fn call(&self, arenas: &Arenas) -> ArenaResult<T> {
        (self.0)(arenas)
    }
}
