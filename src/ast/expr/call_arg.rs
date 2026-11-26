use crate::ast::expr::ExprNode;
use crate::ast::expr::ident::Identifier;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::ArenaItem;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::arena::iterator::ArenaIterator;
use crate::mem::linked::Linked;
use crate::mem::structures::multiple;

#[derive(Debug, Clone)]
pub enum CallArg {
    Ident(Identifier),
    Index(Index<ExprNode>),
}

#[derive(Debug, Clone)]
pub struct CallArgs {
    head: Option<CallArg>,
    tail: Multiple<ExprNode>,
}

#[derive(Debug, Clone)]
pub struct CallArgsIter<Parent> {
    head: Option<CallArg>,
    tail: multiple::Iter<ExprNode, Parent>,
}

impl CallArgs {
    pub fn new(args: Multiple<ExprNode>) -> Self {
        Self { head: None, tail: args }
    }

    pub fn new_receiver(head: Identifier, tail: Multiple<ExprNode>) -> Self {
        Self { head: Some(CallArg::Ident(head)), tail }
    }

    pub fn len(&self) -> u16 {
        u16::from(self.head.is_some()).saturating_add(self.tail.length())
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none() && self.tail.is_empty()
    }

    pub fn iter<Parent>(&self) -> CallArgsIter<Parent> {
        CallArgsIter { head: self.head.clone(), tail: self.tail.iter() }
    }
}

impl<'a, Parent> CallArgsIter<Parent>
where
    Parent: ArenaItem,
    ExprNode: Linked<Parent>,
{
    fn fst_snd<'r, F, A>(
        &'r mut self,
        child_arena: &'a A,
        advance_func: F,
    ) -> (
        impl FnOnce() -> ArenaResult<Option<CallArg>> + 'r,
        impl FnOnce() -> ArenaResult<Option<CallArg>> + 'r,
    )
    where
        'a: 'r,
        A: Arena<ExprNode>,
        F: FnOnce(
                &mut multiple::Iter<ExprNode, Parent>,
                &'a A,
            ) -> ArenaResult<Option<Index<ExprNode>>>
            + 'r,
    {
        let fst = || Ok(self.head.take());
        let tail = &mut self.tail;
        let snd = move || {
            advance_func(tail, child_arena)
                .map(|opt_index| opt_index.map(CallArg::Index))
        };
        (fst, snd)
    }
}

impl<Parent> ArenaIterator for CallArgsIter<Parent>
where
    Parent: ArenaItem,
    ExprNode: Linked<Parent>,
{
    type Input = ExprNode;
    type Output = CallArg;
    type Error = ArenaError;

    fn next_in_arena(
        &mut self,
        arena: &impl Arena<Self::Input>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let (fst, snd) = self.fst_snd(arena, multiple::Iter::next_in_arena);

        if let Some(arg) = fst()? { Ok(Some(arg)) } else { snd() }
    }
}
