#![allow(unused)]

use core::iter;
use core::option;

use fallible_iterator as fiter;
use fallible_iterator::DoubleEndedFallibleIterator;
use fallible_iterator::FallibleIterator;

use crate::ast::expr::ExprNode;
use crate::ast::expr::ident::Identifier;
use crate::ast::expr::linked::ExprArenas;
use crate::mem::ArenaError;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::structures::multiple;

#[derive(Debug, Clone)]
pub enum CallArg {
    Ident(Identifier),
    Index(Index<ExprNode>),
}

#[derive(Debug)]
pub struct CallArgs {
    head: Option<CallArg>,
    tail: Multiple<ExprNode>,
}

#[derive(Debug)]
pub struct CallArgsIter<'a, Arenas>(
    fiter::Chain<
        fiter::Convert<option::IntoIter<ArenaResult<CallArg>>>,
        fiter::Map<
            fiter::Convert<
                multiple::IterChecked<'a, ExprNode, ExprNode, Arenas, Arenas>,
            >,
            fn(Index<ExprNode>) -> ArenaResult<CallArg>,
        >,
    >,
);

impl CallArgs {
    pub fn new(args: Multiple<ExprNode>) -> Self {
        Self { head: None, tail: args }
    }

    pub fn new_receiver(head: Identifier, tail: Multiple<ExprNode>) -> Self {
        Self { head: Some(CallArg::Ident(head)), tail }
    }

    pub fn len(&self) -> u16 {
        u16::from(self.head.is_none()).saturating_add(self.tail.length())
    }

    pub fn iter<'a, Arenas: ExprArenas>(
        &self,
        arenas: &'a Arenas,
    ) -> CallArgsIter<'a, Arenas> {
        CallArgsIter(
            fiter::convert(self.head.clone().map(Ok).into_iter()).chain(
                fiter::convert(self.tail.checked_iter(arenas))
                    .map(|index| Ok(CallArg::Index(index))),
            ),
        )
    }
}

impl<'a, Arenas: ExprArenas> FallibleIterator for CallArgsIter<'a, Arenas> {
    type Item = CallArg;
    type Error = ArenaError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.0.next()
    }
}

impl<'a, Arenas: ExprArenas> DoubleEndedFallibleIterator
    for CallArgsIter<'a, Arenas>
{
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.0.next_back()
    }
}
