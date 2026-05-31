use crate::mem::ArenaResult;
use crate::mem::drop::refs::DropRefs;

pub trait CloneRefs<Arenas>: Sized + DropRefs<Arenas> {
    type CloneRefType;

    fn start_and_result_ref<'a>(
        start: Self,
        arenas: &'a Arenas,
    ) -> ArenaResult<(Self::CloneRefType, Self::DropRefType)>;

    fn process_ref<'a>(
        reference: Self::CloneRefType,
        arenas: &'a Arenas,
    ) -> ArenaResult<impl Iterator<Item = ArenaResult<Self::CloneRefType>> + 'a>;
}
