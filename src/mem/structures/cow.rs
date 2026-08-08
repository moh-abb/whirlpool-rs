use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;

#[derive(Debug)]
/// A type that represents either an owned item, or an index into an allocated
/// item in an arena.
pub enum Cow<T: ArenaItem> {
    Indexed(Index<T>),
    Owned(T),
}

impl<T: ArenaItem> Cow<T> {
    pub fn map<U>(
        &self,
        arena: &impl Arena<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        match self {
            Cow::Indexed(index) => arena.map(index.clone(), func),
            Cow::Owned(item) => Ok(func(item)),
        }
    }

    pub fn map_mut<U>(
        &mut self,
        arena: &impl Arena<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        match self {
            Cow::Indexed(index) => arena.map_mut(index.clone(), func),
            Cow::Owned(item) => Ok(func(item)),
        }
    }
}
