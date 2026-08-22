use chumsky::container::Container;
use heapless::Vec;

trait ConvertConst {
    const _VERIFY: ();
    type Output;
}

struct ConstToU8<const N: usize>;
impl<const N: usize> ConvertConst for ConstToU8<N> {
    const _VERIFY: () = assert!(N <= u8::MAX as usize);
    type Output = u8;
}

pub struct ParserVec<T, const CAP: usize> {
    inner: Vec<T, CAP, <ConstToU8<CAP> as ConvertConst>::Output>,
    exceeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserVecError {
    LimitReached,
}

impl<T, const CAP: usize> Default for ParserVec<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> ParserVec<T, CAP> {
    pub const fn new() -> Self {
        Self { inner: Vec::new(), exceeded: false }
    }

    pub fn as_slice(&self) -> Result<&[T], ParserVecError> {
        if self.exceeded {
            return Err(ParserVecError::LimitReached);
        }

        Ok(self.inner.as_slice())
    }
}

impl<T, const CAP: usize> Container<T> for ParserVec<T, CAP> {
    fn push(&mut self, item: T) {
        let push_result = self.inner.push(item);
        self.exceeded = push_result.is_err();
    }
}
