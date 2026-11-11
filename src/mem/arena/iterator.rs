use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;

pub trait ArenaIterator {
    type Input: ArenaItem;
    type Output;
    type Error: From<ArenaError>;

    fn next_in_arena(
        &mut self,
        arena: &impl Arena<Self::Input>,
    ) -> Result<Option<Self::Output>, Self::Error>;
}
