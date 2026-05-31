use core::cmp::Ordering;
use core::ops::ControlFlow;

use crate::mem::ArenaResult;
use crate::mem::Vec;

pub mod refs;

pub fn cmp_in_arenas<T: refs::OrdRefs<ArenasX, ArenasY>, ArenasX, ArenasY>(
    start_x: T,
    start_y: T,
    arenas_x: &ArenasX,
    arenas_y: &ArenasY,
) -> ArenaResult<Ordering> {
    // Use vector (to be replaced with stack-allocated version) to avoid stack
    // overflow.
    let mut references = Vec::new();
    let (start_ref_x, start_ref_y) = T::start_refs(start_x, start_y);
    references.push((start_ref_x, start_ref_y));
    while let Some((cur_ref_x, cur_ref_y)) = references.pop() {
        // On an invalid reference, simply break with the error (with no need
        // for further recovery)
        let opt_next_refs =
            T::process_ref(cur_ref_x, cur_ref_y, arenas_x, arenas_y)?;
        match opt_next_refs {
            // When the current two references are known to be equal, continue
            ControlFlow::Break(Ordering::Equal) => continue,
            // When one reference is greater or smaller, the entire comparison
            // is also lexicographically larger or smaller.
            ControlFlow::Break(ordering) => return Ok(ordering),
            // Continue with the remaining references.
            ControlFlow::Continue(mut next_refs) => {
                next_refs.try_for_each(|(next_ref_x, next_ref_y)| {
                    references.push((next_ref_x?, next_ref_y?));
                    Ok(())
                })?
            }
        }
    }
    // When all elements are known to be equal, the traversal should give
    Ok(Ordering::Equal)
}
