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
