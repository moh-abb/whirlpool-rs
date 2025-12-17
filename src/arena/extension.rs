use super::Arena;
use super::ArenaItem;
use super::error::ArenaResult;
use super::index::Index;

/// A helper trait for constructing a value from an array of indices after
/// allocating (appending) values of that array's length into this [Arena].
#[allow(unused)]
pub trait AllocMany<T: ArenaItem>: Arena<T> {
    /// Constructs the given value after allocating each of the array of values
    /// and obtaining all the corresponding [Index]es that
    /// key into the array.
    fn alloc_many<const N: usize, U, F>(
        &mut self,
        values: [T; N],
        constructor: impl FnOnce([Index<T>; N]) -> U,
    ) -> ArenaResult<U> {
        let mut result = [const { None }; N];
        values
            .into_iter()
            .enumerate()
            .try_for_each(|(index, value)| {
                result[index] = Some(self.alloc(value)?);
                Ok(())
            })?;
        Ok(constructor(result.map(Option::unwrap)))
    }
}

impl<T: ArenaItem, A: Arena<T> + ?Sized> AllocMany<T> for A {}

/// A helper trait for keying into the `Arena` to get a temporary reference to
/// the corresponding item, and mapping it to a specified type as given by the
/// function passed in.
/// Because the [Arena] is accessed immutably, for memory safety, the function
/// needs to operate with a temporary value (lifetime) of `&T`.
#[allow(unused)]
pub trait Inspect<T: ArenaItem>: Arena<T> {
    fn inspect<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&T) -> U,
    ) -> ArenaResult<U> {
        let x = self.take(index.clone())?;
        let y = func(&x);
        self.insert(index, x)?;
        Ok(y)
    }

    #[cfg(test)]
    fn inspect_mut<U>(
        &self,
        index: Index<T>,
        func: impl FnOnce(&mut T) -> U,
    ) -> ArenaResult<U> {
        let mut x = self.take(index.clone())?;
        let y = func(&mut x);
        self.insert(index, x)?;
        Ok(y)
    }
}

impl<T: ArenaItem, A: Arena<T> + ?Sized> Inspect<T> for A {}
