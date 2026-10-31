use fallible_iterator::FallibleIterator;

use crate::interpreter::error::PatternInterpreterError;

pub trait Generator
where
    Self: FallibleIterator<Error = PatternInterpreterError>,
{
}

impl<T, I> Generator for I where
    Self: FallibleIterator<Item = T, Error = PatternInterpreterError>
{
}

/// Adapts an iterator that returns values of type `T` into values of type
/// `Result::<T, E>::Ok(T)`.
#[derive(Debug)]
pub struct GenPure<I: Iterator>(I);

impl<I: Iterator> GenPure<I> {
    pub fn new(iter: I) -> Self {
        Self(iter)
    }
}

impl<I: Iterator> FallibleIterator for GenPure<I> {
    type Item = I::Item;
    type Error = PatternInterpreterError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.0.next())
    }
}

/// Creates a custom implementation of [core::iter::Map]-style iterators
/// with two fields: the iterator, and a state.
/// Expects a block that returns `Result<Option<Output>>`.
/// (The definition is inspired by the `combine::parser!` macro.)
/// Usage:
/// ```rust,ignore
/// gen_custom! {
///     struct PlusOne[Num](
///         value: core::iter::Once<Num>,
///         inc: Num,
///     ) -> usize
///         where struct []
///         where iter [usize: From<Num>, Num: core::ops::Add<Output = Num>,]
///     {
///         Ok(Some(usize::from(
///             value.map(|n| n + *inc)
///                 .ok_or(todo!())?
///         )))
///     }
/// };
/// ```
#[macro_export]
macro_rules! gen_custom {
    {
        $(#[$attr:meta])*
        $v:vis struct $name:ident [$($types:tt)*] (
            $iter_name:ident: $iter_type:ty,
            $state_name:ident: $state_type:ty $(,)?
        ) -> $output_type:ty
        where struct [$($struct_clauses:tt)*]
        where iter [$($iter_clauses:tt)*]
        $fun_blk:block
    } => {
        $(#[$attr])*
        $v struct $name<$($types)*>(
            $iter_type,
            $state_type,
        ) where $($struct_clauses)*;
        impl<$($types)*> fallible_iterator::FallibleIterator
            for $name<$($types)*>
            where $($iter_clauses)* $($struct_clauses)*
        {
            type Item = $output_type;
            type Error = crate::interpreter::error::PatternInterpreterError;
            fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
                let $iter_name = self.0.next();
                let $state_name = &mut self.1;
                $fun_blk
            }
        }
    };
}
