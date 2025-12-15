//! Error messages for different arena allocation failure cases.

use core::fmt::Debug;
use core::fmt::Display;

/// The message to use when allocation fails due to a lack of available space.
const LIMIT_REACHED: &str = "[Arena::alloc]: Arena limit reached";
/// The message to use during an attempt to insert a
/// value into a slot ([Option]) which is already full, i.e. [Some].
const EXPECTED_FREE_SLOT: &str =
    "[Arena::alloc]: Arena slot should have been empty at the given index";
/// The message to use during an attempt to access a
/// slot ([Option]) for which the given [super::Index] is too large for
/// the [super::Arena].
const INDEX_OUT_OF_BOUNDS: &str =
    "[Arena]: Index out of bounds";
/// The message to use during an attempt to access a
/// value into a slot ([Option]) which is empty, i.e. [None].
const EXPECTED_FULL_SLOT: &str =
    "[Arena::take]: Arena slot should have been full at the given index";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum ArenaError {
    IndexOutOfBounds,
    LimitReached,
    ExpectedFreeSlot,
    ExpectedFullSlot,
}

pub type ArenaResult<T> = Result<T, ArenaError>;

impl Display for ArenaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::IndexOutOfBounds => INDEX_OUT_OF_BOUNDS,
            Self::LimitReached => LIMIT_REACHED,
            Self::ExpectedFreeSlot => EXPECTED_FREE_SLOT,
            Self::ExpectedFullSlot => EXPECTED_FULL_SLOT,
        };
        f.write_str(msg)
    }
}
