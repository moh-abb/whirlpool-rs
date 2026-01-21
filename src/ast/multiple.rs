use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::arena::chain::Chain;
use crate::arena::index::Index;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Multiple<Item: ArenaItem> {
    pub length: u16,
    pub index: Index<Chain<Item>>,
}

impl<Item: ArenaItem> Multiple<Item> {
    pub fn map_reduce<Acc>(
        &self,
        arena: &impl Arena<Chain<Item>>,
        init: Acc,
        mut fold: impl FnMut(Acc, Index<Item>) -> Acc,
    ) -> Acc {
        let mut result = init;
        let mut cur_index = self.index.clone();
        loop {
            let cloned_chain = arena
                .inspect(cur_index, Clone::clone)
                .unwrap();
            match cloned_chain {
                Chain::Cons { head, tail } => {
                    cur_index = tail;
                    result = fold(result, head);
                }
                Chain::Nil => {
                    break result;
                }
            }
        }
    }

    #[allow(unused)]
    pub fn verify_length(&self, arena: &impl Arena<Chain<Item>>) {
        let manually_counted_length =
            self.map_reduce(arena, 0, |sum, _| sum + 1);
        assert_eq!(manually_counted_length, self.length);
    }
}
