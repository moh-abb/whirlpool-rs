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
const INDEX_OUT_OF_BOUNDS: &str = "[Arena]: Index out of bounds";
/// The message to use during an attempt to access a
/// value into a slot ([Option]) which is empty, i.e. [None].
const EXPECTED_FULL_SLOT: &str =
    "[Arena::take]: Arena slot should have been full at the given index";
/// The message to use when an arena using a [core::cell::RefCell] is
/// borrowed twice.
const INVALID_BORROW: &str =
    "[Arena]: Attempted to borrow arena violating borrowing rules";
/// The message to use when attempting to access an element which is expected
/// to be linked, and have a parent, but none was found.
const EXPECTED_PARENT: &str = "[Arena]: Expected parent for the given index";
/// The message to use when attempting to access an element which is expected
/// to be linked, and have children, but none were found.
const EXPECTED_CHILDREN: &str =
    "[Arena]: Expected children for the given index";
/// The message to use when attempting to access an element which is expected
/// to be linked, and have a sibling on a given end, but it was not found.
const EXPECTED_SIBLING: &str =
    "[Arena]: Expected a sibling for the given index";
/// The message to use when an invariant during traversal has been broken.
/// This should not be used as the main source of error information; it only
/// serves as a placeholder for a situation that is logically unreachable and
/// should be complemented by more verbose error propagation.
const INVARIANT_BROKEN: &str = "[Arena]: Invariant broken during traversal";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArenaError {
    IndexOutOfBounds,
    LimitReached,
    ExpectedFreeSlot,
    ExpectedFullSlot,
    InvalidBorrow,
    ExpectedParent,
    ExpectedChildren,
    ExpectedSibling,
    InvariantBroken,
}

pub type ArenaResult<T> = Result<T, ArenaError>;

impl Display for ArenaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::IndexOutOfBounds => INDEX_OUT_OF_BOUNDS,
            Self::LimitReached => LIMIT_REACHED,
            Self::ExpectedFreeSlot => EXPECTED_FREE_SLOT,
            Self::ExpectedFullSlot => EXPECTED_FULL_SLOT,
            Self::InvalidBorrow => INVALID_BORROW,
            Self::ExpectedParent => EXPECTED_PARENT,
            Self::ExpectedChildren => EXPECTED_CHILDREN,
            Self::ExpectedSibling => EXPECTED_SIBLING,
            Self::InvariantBroken => INVARIANT_BROKEN,
        };
        f.write_str(msg)
    }
}
