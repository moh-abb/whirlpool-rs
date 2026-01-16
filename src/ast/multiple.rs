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
    #[allow(unused)]
    pub fn verify_length(&self, arena: &impl Arena<Chain<Item>>) {
        let mut manually_counted_length = 0;
        let mut cur_index = self.index.clone();
        loop {
            let cloned_chain = arena
                .inspect(cur_index, Clone::clone)
                .unwrap();
            match cloned_chain {
                Chain::Cons { head: _, tail } => {
                    cur_index = tail;
                    manually_counted_length += 1;
                }
                Chain::Nil => {
                    assert_eq!(manually_counted_length, self.length);
                    break;
                }
            }
        }
    }
}
