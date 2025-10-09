use core::num::NonZeroU8;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Number(pub u16);

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Note {
    pub note: Letter,
    pub octave: NonZeroU8,
}

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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum NoteUnit {
    Letter(Letter),
    Number(Number),
    Frequency(Frequency),
}
