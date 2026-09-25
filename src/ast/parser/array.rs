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

#[derive(Debug, Clone)]
pub struct ParserVec<T, const CAP: usize> {
    inner: Vec<T, CAP, <ConstToU8<CAP> as ConvertConst>::Output>,
    exceeded: bool,
    _verify: (),
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
        Self {
            inner: Vec::new(),
            exceeded: false,
            _verify: <ConstToU8<CAP> as ConvertConst>::_VERIFY,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn as_slice(&self) -> Result<&[T], ParserVecError> {
        if self.exceeded {
            return Err(ParserVecError::LimitReached);
        }

        Ok(self.inner.as_slice())
    }
}

impl<T, const CAP: usize> Extend<T> for ParserVec<T, CAP> {
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        iter.into_iter().for_each(|item| {
            if self.exceeded {
                return;
            }
            self.exceeded = self.inner.push(item).is_err();
        });
    }
}
