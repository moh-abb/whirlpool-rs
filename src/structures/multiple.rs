use crate::arena::Arena;
use crate::arena::ArenaItem;
use crate::structures::chain::Chain;
use crate::structures::index::Index;

type StartEnd<Item> = Option<(Index<Chain<Item>>, Index<Chain<Item>>)>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Multiple<Item: ArenaItem> {
    length: u16,
    start_end: StartEnd<Item>,
}

impl<Item: ArenaItem> Multiple<Item> {
    #[allow(unused)]
    pub fn new_nonempty(
        length: u16,
        start: Index<Chain<Item>>,
        end: Index<Chain<Item>>,
    ) -> Self {
        Self { length, start_end: Some((start, end)) }
    }

    #[allow(unused)]
    pub fn new_empty() -> Self {
        Self { length: 0, start_end: None }
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn start_end(&self) -> StartEnd<Item> {
        self.start_end.clone()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.start_end.is_none(), self.length == 0);
        self.length == 0
    }

    pub fn push_back(
        &mut self,
        arena: &impl Arena<Chain<Item>>,
        elem_index: Index<Chain<Item>>,
    ) {
        // The element should not be connected to anything.
        let elem_is_singleton = arena
            .inspect(elem_index.clone(), |e| e.1.is_none() && e.2.is_none())
            .unwrap();
        debug_assert!(elem_is_singleton);
        let new_start_end = if self.is_empty() {
            // The list is empty; update the chain to a singleton.
            (elem_index.clone(), elem_index.clone())
        } else {
            // The list is not empty; push to the end.
            let (start, end) = self.start_end.clone().unwrap();
            let new_end = elem_index.clone();
            let link_end_to_new_end = |end_elem: &mut Chain<Item>| {
                let Chain(_index, _prev, next) = end_elem;
                debug_assert!(next.is_none());
                next.replace(new_end.clone());
            };
            let link_new_end_to_end = |new_end_elem: &mut Chain<Item>| {
                let Chain(_index, prev, _next) = new_end_elem;
                debug_assert!(prev.is_none());
                prev.replace(end.clone());
            };
            arena
                .inspect_mut(end.clone(), link_end_to_new_end)
                .unwrap();
            arena
                .inspect_mut(new_end.clone(), link_new_end_to_end)
                .unwrap();
            (start, new_end)
        };
        // Update the start and end indices, and the length
        self.length += 1;
        let _ = self.start_end.insert(new_start_end);
    }

    pub fn pop_back(
        &mut self,
        arena: &impl Arena<Chain<Item>>,
    ) -> Option<Index<Chain<Item>>> {
        let Some((start, end)) = self.start_end.clone() else {
            debug_assert_eq!(self.length, 0);
            return None;
        };
        let end_prev = arena
            .inspect_mut(end.clone(), |end_elem| {
                let Chain(_end_index, end_prev, _end_next) = end_elem;
                debug_assert!(_end_next.is_none());
                // Unlink the end element from the element before it.
                end_prev.take()
            })
            .unwrap();
        let new_start_end = if let Some(prev) = end_prev {
            // The list was originally longer than a singleton.
            let unlink_end = |prev_elem: &mut Chain<Item>| {
                let Chain(_prev_index, _prev_prev, prev_next) = prev_elem;
                debug_assert_eq!(
                    prev_next.clone().map(usize::from),
                    Some(usize::from(end.clone()))
                );
                prev_next.take();
            };
            arena
                .inspect_mut(prev.clone(), unlink_end)
                .unwrap();
            Some((start, prev))
        } else {
            // The list was originally a singleton.
            None
        };
        // Update the start and end indices, and the length
        self.length -= 1;
        self.start_end = new_start_end;
        Some(end)
    }

    fn fold<Acc>(
        &self,
        arena: &impl Arena<Chain<Item>>,
        init: Acc,
        mut fold: impl FnMut(Acc, Index<Item>) -> Acc,
        start_index: impl FnOnce(
            Index<Chain<Item>>,
            Index<Chain<Item>>,
        ) -> Index<Chain<Item>>,
        mut next_index: impl FnMut(
            Option<Index<Chain<Item>>>,
            Option<Index<Chain<Item>>>,
        ) -> Option<Index<Chain<Item>>>,
    ) -> Acc {
        let mut result = init;
        let Some((start, end)) = self.start_end.clone() else {
            return result;
        };
        let mut cur_pos = start_index(start, end);
        loop {
            let chain = arena
                .inspect(cur_pos, Clone::clone)
                .unwrap();
            let Chain(index, prev, next) = chain;
            result = fold(result, index);
            let Some(next_pos) = next_index(prev, next) else {
                break result;
            };
            cur_pos = next_pos;
        }
    }

    #[allow(unused)]
    pub fn fold_right<Acc>(
        &self,
        arena: &impl Arena<Chain<Item>>,
        init: Acc,
        fold: impl FnMut(Acc, Index<Item>) -> Acc,
    ) -> Acc {
        self.fold(arena, init, fold, |_, end| end, |prev, _| prev)
    }

    #[allow(unused)]
    pub fn fold_left<Acc>(
        &self,
        arena: &impl Arena<Chain<Item>>,
        init: Acc,
        fold: impl FnMut(Acc, Index<Item>) -> Acc,
    ) -> Acc {
        self.fold(arena, init, fold, |start, _| start, |_, next| next)
    }

    #[allow(unused)]
    pub fn verify_length(&self, arena: &impl Arena<Chain<Item>>) {
        let manually_counted_length =
            self.fold_left(arena, 0, |sum, _| sum + 1);
        assert_eq!(manually_counted_length, self.length);
        // Chase the pointers to ensure consistency.
        match self.start_end() {
            None => assert!(self.is_empty()),
            Some((start, end)) => {
                let mut from_left = start.clone();
                (0..manually_counted_length - 1).for_each(|_| {
                    let chain = arena
                        .inspect(from_left.clone(), Clone::clone)
                        .unwrap();
                    let Chain(_index, _prev, next) = chain;
                    from_left = next.unwrap();
                });
                assert_eq!(
                    usize::from(from_left.clone()),
                    usize::from(end.clone())
                );
                let mut from_right = end.clone();
                (0..manually_counted_length - 1).for_each(|_| {
                    let chain = arena
                        .inspect(from_right.clone(), Clone::clone)
                        .unwrap();
                    let Chain(_index, prev, _next) = chain;
                    from_right = prev.unwrap();
                });
                assert_eq!(
                    usize::from(from_right.clone()),
                    usize::from(start.clone())
                );
            }
        }
    }
}
