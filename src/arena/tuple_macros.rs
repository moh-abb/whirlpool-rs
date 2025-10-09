/// A way to create a right-associated [RightTuple] of types. Used to implement
/// the [ArenaHandler::Indices] type, by passing in a nonempty list of types
/// (separated by commas) for which indices of that type are referred to in the
/// `Self` type.
#[macro_export]
macro_rules! handle_indices {
    ($type:path, $($types:path),+) => {
        ($type, handle_indices!($($types),+))
    };
    ($type:path) => {
        ($type, ())
    };
}

/// A way to create a right-associated [RightTuple] of dynamically-dispatched
/// [Arena] references. Used to implement the [ArenaHandler::DynArenas] type,
/// by passing in the same lifetime argument as in [ArenaHandler::DynArenas],
/// followed by the same nonempty list of types (separated by commas) as in
/// [handle_indices].
#[macro_export]
macro_rules! handle_dyn_arenas {
    ($l:lifetime, $type:path, $($types:path),+) => {
        (
            &'a dyn $crate::arena::Arena<$type>,
            handle_dyn_arenas!($l, $($types),+),
        )
    };
    ($l:lifetime, $type:path) => {
        (
            &'a dyn $crate::arena::Arena<$type>,
            (),
        )
    };
}
