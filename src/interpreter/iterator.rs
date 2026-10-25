use core::marker::PhantomData;

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
pub struct GenPure<I: Iterator, E>(I, PhantomData<E>);

impl<I: Iterator, E> GenPure<I, E> {
    pub fn new(iter: I) -> Self {
        Self(iter, PhantomData)
    }
}

impl<I: Iterator, E> FallibleIterator for GenPure<I, E> {
    type Item = I::Item;
    type Error = E;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.0.next())
    }
}
