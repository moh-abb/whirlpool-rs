use crate::mem::ArenaResult;

pub trait DropRefs<Arenas>: Sized {
    type DropRefType;

    fn start_ref(start: Self) -> Self::DropRefType;

    fn process_ref<'a>(
        reference: Self::DropRefType,
        arenas: &'a Arenas,
    ) -> ArenaResult<impl Iterator<Item = ArenaResult<Self::DropRefType>> + 'a>;
}
