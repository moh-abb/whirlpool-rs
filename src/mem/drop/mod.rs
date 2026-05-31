use crate::mem::ArenaResult;
use crate::mem::Vec;

pub mod refs;

pub fn drop_in_arenas_from_start_ref<T: refs::DropRefs<Arenas>, Arenas>(
    start_ref: T::DropRefType,
    arenas: &Arenas,
) -> ArenaResult<()> {
    // Use vector (to be replaced with stack-allocated version) to avoid stack
    // overflow.
    let mut references = Vec::new();
    references.push(start_ref);
    while let Some(cur_ref) = references.pop() {
        let Ok(mut next_refs) = T::process_ref(cur_ref, arenas) else {
            // We have reached an invalid reference.
            // We could use ? to terminate completely, but instead attempt to
            // recover by dropping the remaining references.
            continue;
        };
        next_refs.try_for_each(|next_ref| {
            references.push(next_ref?);
            Ok(())
        })?;
    }
    Ok(())
}

pub fn drop_in_arenas<T: refs::DropRefs<Arenas>, Arenas>(
    start: T,
    arenas: &Arenas,
) -> ArenaResult<()> {
    drop_in_arenas_from_start_ref::<T, Arenas>(T::start_ref(start), arenas)
}
