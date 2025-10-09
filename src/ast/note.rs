#[cfg(test)]
use proptest_derive::Arbitrary;

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Letter {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Frequency(pub f32);
