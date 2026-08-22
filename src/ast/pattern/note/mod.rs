use core::num::NonZeroU8;

use crate::ast::CycleTime;

pub mod arbitrary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Note {
    letter: NoteLetter,
    octave: NonZeroU8,
}

impl Note {
    /// Strudel uses the default octave of 3.
    pub const DEFAULT_OCTAVE: NonZeroU8 = NonZeroU8::new(3).unwrap();

    pub const fn new(letter: NoteLetter) -> Self {
        Self { letter, octave: Self::DEFAULT_OCTAVE }
    }
}

/// Represents the type of note literals. In the AST, we have only one canonical
/// form of a note, so the "flat" / "sharp" variants are encoded equally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum NoteLetter {
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

#[derive(
    derive_more::Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[display("{_0}")]
pub struct NoteNumber(pub CycleTime);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoteUnit {
    WithOctave(Note),
    Number(NoteNumber),
}
