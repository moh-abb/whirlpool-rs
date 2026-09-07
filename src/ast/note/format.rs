use core::fmt;

use crate::ast::display::AstDisplay;
use crate::ast::display::AstDisplayFlags;
use crate::ast::note::Note;
use crate::ast::note::NoteLetter;
use crate::ast::note::NoteUnit;
use crate::mem::debug_unwrap;

#[derive(derive_more::From, Debug)]
pub enum DisplayNoteUnitError {
    FormatErr(fmt::Error),
    LetterMapFailed,
}

impl fmt::Display for NoteUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_unwrap(<Self as AstDisplay>::fmt(
            self,
            f,
            &AstDisplayFlags::default(),
        ));
        Ok(())
    }
}

impl AstDisplay for NoteUnit {
    type Error = DisplayNoteUnitError;

    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
        _flags: &AstDisplayFlags,
    ) -> Result<(), Self::Error> {
        match self {
            NoteUnit::WithOctave(Note { letter, octave }) => {
                let letter_map = [
                    (NoteLetter::A, "a"),
                    (NoteLetter::B, "b"),
                    (NoteLetter::C, "c"),
                    (NoteLetter::D, "d"),
                    (NoteLetter::E, "e"),
                    (NoteLetter::F, "f"),
                    (NoteLetter::G, "g"),
                    (NoteLetter::ASharp, "a#"),
                    (NoteLetter::CSharp, "c#"),
                    (NoteLetter::DSharp, "d#"),
                    (NoteLetter::FSharp, "f#"),
                    (NoteLetter::GSharp, "g#"),
                ];
                let letter_str = letter_map
                    .iter()
                    .find_map(|(cur_letter, cur_str)| {
                        (cur_letter == letter).then_some(cur_str)
                    })
                    .ok_or(DisplayNoteUnitError::LetterMapFailed)?;
                write!(f, "{letter_str}{octave}")?;
            }
            NoteUnit::Number(note_number) => write!(f, "{note_number}")?,
        }
        Ok(())
    }
}
