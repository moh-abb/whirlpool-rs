use crate::mem::ArenaResult;

pub trait DropRefs<Arenas>: Sized {
    type Reference;

    fn start_ref(start: Self) -> Self::Reference;

    fn process_ref<'a>(
        reference: Self::Reference,
        arenas: &'a Arenas,
    ) -> ArenaResult<impl Iterator<Item = ArenaResult<Self::Reference>> + 'a>;
}
