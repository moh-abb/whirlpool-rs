use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::Vec;
use crate::mem::drop::drop_in_arenas_from_start_ref;

pub mod refs;

pub fn clone_in_arenas<T: refs::CloneRefs<Arenas>, Arenas>(
    start: T,
    arenas: &Arenas,
) -> ArenaResult<T::DropRefType> {
    // Use vector (to be replaced with stack-allocated version) to avoid stack
    // overflow.
    let mut references = Vec::new();
    // The end reference is the reference to the cloned result.
    let (start_ref, result_ref) = T::start_and_result_ref(start, arenas)?;
    let mut opt_result_ref = Some(result_ref);
    references.push(start_ref);
    while let Some(cur_ref) = references.pop() {
        let mut drop_remaining = |err: ArenaError| {
            // On an invalid reference, recover by dropping from the start
            // (ignoring any errors along the way).
            let Some(result_ref) = opt_result_ref.take() else {
                // The result reference has already been used.
                return ArenaResult::Ok(());
            };
            let _ =
                drop_in_arenas_from_start_ref::<T, Arenas>(result_ref, arenas);
            return ArenaResult::Err(err);
        };

        match <T as refs::CloneRefs<Arenas>>::process_ref(cur_ref, arenas) {
            Err(err) => drop_remaining(err)?,
            Ok(next_refs) => {
                for opt_next_ref in next_refs {
                    match opt_next_ref {
                        Ok(next_ref) => references.push(next_ref),
                        Err(err) => drop_remaining(err)?,
                    }
                }
            }
        }
    }
    Ok(opt_result_ref.take().unwrap())
}
