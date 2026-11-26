use core::num::NonZeroU8;

use crate::ast::note::Note;
use crate::ast::note::NoteLetter;

#[derive(Debug)]
pub enum ResolveNoteError {
    EmptyInput,
    NonUtf8Input,
    UnexpectedFirst(u8),
    InvalidOctaveStartingWith(u8),
    UnexpectedFlat(NoteLetter),
    UnexpectedSharp(NoteLetter),
    Unreachable,
}

impl Note {
    pub fn resolve(bytes: &[u8]) -> Result<Self, ResolveNoteError> {
        // Tries to parse `xs` as an octave.
        let slice_to_octave = |xs: &[u8]| {
            let Some(y) = xs.first() else {
                return Ok(Self::DEFAULT_OCTAVE);
            };
            let xs_str = str::from_utf8(xs)
                .map_err(|_| ResolveNoteError::NonUtf8Input)?;
            xs_str
                .parse::<NonZeroU8>()
                .map_err(|_| ResolveNoteError::InvalidOctaveStartingWith(*y))
        };

        let Some((x, xs)) = bytes.split_first() else {
            return Err(ResolveNoteError::EmptyInput);
        };

        let first_parse_letter = match x.to_ascii_uppercase() {
            b'C' => NoteLetter::C,
            b'D' => NoteLetter::D,
            b'E' => NoteLetter::E,
            b'F' => NoteLetter::F,
            b'G' => NoteLetter::G,
            b'A' => NoteLetter::A,
            b'B' => NoteLetter::B,
            _ => return Err(ResolveNoteError::UnexpectedFirst(*x)),
        };

        let Some((y, ys)) = xs.split_first() else {
            return Ok(Self::new(first_parse_letter));
        };

        let (second_parse_letter, zs) = match y.to_ascii_lowercase() {
            b'b' => {
                let letter = match first_parse_letter {
                    NoteLetter::D => NoteLetter::CSharp,
                    NoteLetter::E => NoteLetter::DSharp,
                    NoteLetter::G => NoteLetter::FSharp,
                    NoteLetter::A => NoteLetter::GSharp,
                    NoteLetter::B => NoteLetter::ASharp,
                    NoteLetter::C | NoteLetter::F => {
                        return Err(ResolveNoteError::UnexpectedFlat(
                            first_parse_letter,
                        ));
                    }
                    _ => return Err(ResolveNoteError::Unreachable),
                };
                (letter, ys)
            }
            b'#' => {
                let letter = match first_parse_letter {
                    NoteLetter::C => NoteLetter::CSharp,
                    NoteLetter::D => NoteLetter::DSharp,
                    NoteLetter::F => NoteLetter::FSharp,
                    NoteLetter::G => NoteLetter::GSharp,
                    NoteLetter::A => NoteLetter::ASharp,
                    NoteLetter::E | NoteLetter::B => {
                        return Err(ResolveNoteError::UnexpectedSharp(
                            first_parse_letter,
                        ));
                    }
                    _ => return Err(ResolveNoteError::Unreachable),
                };
                (letter, ys)
            }
            _ => (first_parse_letter, xs),
        };

        Ok(Self { letter: second_parse_letter, octave: slice_to_octave(zs)? })
    }
}
