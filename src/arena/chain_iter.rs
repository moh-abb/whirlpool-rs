use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem;

use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::chain::ChainOrIndex;
use crate::arena::extension::Inspect;

enum ItemFunction<T, U, RefFunc, MutFunc> {
    ByRef(RefFunc, PhantomData<T>, PhantomData<U>),
    ByMut(MutFunc, PhantomData<T>, PhantomData<U>),
}

impl<T, U, RefFunc, MutFunc> Debug for ItemFunction<T, U, RefFunc, MutFunc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ByRef(_, _, _) => f.write_str("ByRef"),
            Self::ByMut(_, _, _) => f.write_str("ByMut"),
        }
    }
}

#[derive(Debug)]
struct Iter<'a, T, U, HeadA: ?Sized, ChainA: ?Sized, RefFunc, MutFunc> {
    chain_or_index: ChainOrIndex<T>,
    head_arena: &'a HeadA,
    chain_arena: &'a ChainA,
    func: ItemFunction<T, U, RefFunc, MutFunc>,
}

impl<T: ArenaItem> Chain<T> {
    #[allow(unused)]
    pub fn iter<'a, U, HeadA, ChainA>(
        &'a self,
        head_arena: &'a HeadA,
        chain_arena: &'a ChainA,
        func: impl FnMut(&T) -> U,
    ) -> impl Iterator<Item = U>
    where
        HeadA: Arena<T> + ?Sized,
        ChainA: Arena<Chain<T>> + ?Sized,
    {
        let chain_or_index = ChainOrIndex::Chain(self.clone());
        Iter {
            chain_or_index,
            head_arena,
            chain_arena,
            func: ItemFunction::<_, _, _, fn(&mut T) -> U>::ByRef(
                func,
                PhantomData,
                PhantomData,
            ),
        }
    }

    #[allow(unused)]
    pub fn iter_mut<'a, U, HeadA, ChainA>(
        &'a self,
        head_arena: &'a HeadA,
        chain_arena: &'a ChainA,
        func: impl FnMut(&mut T) -> U,
    ) -> impl Iterator<Item = U>
    where
        HeadA: Arena<T>,
        ChainA: Arena<Chain<T>>,
    {
        let chain_or_index = ChainOrIndex::Chain(self.clone());
        Iter {
            chain_or_index,
            head_arena,
            chain_arena,
            func: ItemFunction::<_, _, fn(&T) -> U, _>::ByMut(
                func,
                PhantomData,
                PhantomData,
            ),
        }
    }
}

impl<'a, T, U, HeadA, ChainA, RefFunc, MutFunc> Iterator
    for Iter<'a, T, U, HeadA, ChainA, RefFunc, MutFunc>
where
    T: ArenaItem,
    HeadA: Arena<T> + ?Sized,
    ChainA: Arena<Chain<T>> + ?Sized,
    RefFunc: FnMut(&T) -> U,
    MutFunc: FnMut(&mut T) -> U,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { chain_or_index, head_arena, chain_arena, func } = self;
        let (head, tail) = match chain_or_index {
            ChainOrIndex::Chain(chain) => {
                let Chain::Cons { head, tail } = chain else {
                    return None;
                };
                (head.clone(), tail.clone())
            }
            ChainOrIndex::Index(index) => {
                let chain = chain_arena.inspect(index.clone(), Chain::clone);
                let Ok(Chain::Cons { head, tail }) = chain else {
                    return None;
                };
                (head, tail)
            }
        };
        let inspected = match func {
            ItemFunction::ByRef(f, _, _) => {
                <HeadA as Inspect<T>>::inspect(*head_arena, head, f)
            }
            ItemFunction::ByMut(f, _, _) => {
                <HeadA as Inspect<T>>::inspect_mut(*head_arena, head, f)
            }
        };
        let head_value = inspected.ok()?;
        let next_chain_or_index = ChainOrIndex::Index(tail);
        let _ = mem::replace(chain_or_index, next_chain_or_index);
        Some(head_value)
    }
}
