#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Frequency(pub f32);
