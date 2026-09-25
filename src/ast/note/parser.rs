use core::num::NonZeroU8;
use core::num::ParseIntError;
use core::str::FromStr;

use combine::Parser;
use combine::attempt;
use combine::choice;
use combine::many;
use combine::satisfy;
use combine::token;

use crate::ast::note::Note;
use crate::ast::note::NoteLetter;
use crate::ast::note::NoteNumber;
use crate::ast::note::NoteUnit;
use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::array::ParserVec;
use crate::ast::parser::array::ParserVecError;
use crate::ast::time::CycleTime;
use crate::ast::time::parser::ParseCycleTimeError;

#[derive(derive_more::From, Debug, Clone)]
pub enum ParseNoteUnitError {
    CycleTimeErr(ParseCycleTimeError),
    ParseOctaveErr(ParseOctaveError),
}

#[derive(derive_more::From, Debug, Clone)]
pub enum ParseOctaveError {
    FromIntErr(ParseIntError),
    StringBufferErr(ParserVecError),
    Utf8Err,
}

const OCTAVE_STRING_CAPACITY: usize = 8;
type OctaveString = ParserVec<u8, OCTAVE_STRING_CAPACITY>;

impl NoteNumber {
    fn parser<I: AstInput>()
    -> impl AstParser<I, Result<Self, ParseNoteUnitError>> + Clone {
        CycleTime::parser().map(|x| {
            x.map(NoteNumber)
                .map_err(ParseNoteUnitError::CycleTimeErr)
        })
    }
}

impl NoteLetter {
    fn parser<I: AstInput>() -> impl AstParser<I, Self> + Clone {
        let aug_notes = [
            (b'c', b'#', NoteLetter::CSharp),
            (b'd', b'b', NoteLetter::CSharp),
            (b'd', b'#', NoteLetter::DSharp),
            (b'e', b'b', NoteLetter::DSharp),
            (b'f', b'#', NoteLetter::FSharp),
            (b'g', b'b', NoteLetter::FSharp),
            (b'g', b'#', NoteLetter::GSharp),
            (b'a', b'b', NoteLetter::GSharp),
            (b'a', b'#', NoteLetter::ASharp),
            (b'b', b'b', NoteLetter::ASharp),
        ];
        let notes = [
            (b'c', NoteLetter::C),
            (b'd', NoteLetter::D),
            (b'e', NoteLetter::E),
            (b'f', NoteLetter::F),
            (b'g', NoteLetter::G),
            (b'a', NoteLetter::A),
            (b'b', NoteLetter::B),
        ];
        let ignore_case =
            |byte: u8| token(byte).or(token(byte.to_ascii_uppercase()));
        let aug_note_parsers = aug_notes.map(|(note_byte, aug_byte, res)| {
            (ignore_case(note_byte), ignore_case(aug_byte)).map(move |_| res)
        });
        let note_map_parsers = notes.map(|(note_char, res)| {
            token(note_char)
                .or(token(note_char.to_ascii_uppercase()))
                .map(move |_| res)
        });
        choice((
            choice(aug_note_parsers.map(attempt)),
            choice(note_map_parsers),
        ))
    }
}

impl Note {
    fn parser<I: AstInput>()
    -> impl AstParser<I, Result<Self, ParseNoteUnitError>> + Clone {
        let parse_octave = |octave_str: OctaveString| {
            if octave_str.is_empty() {
                return Ok(Note::DEFAULT_OCTAVE);
            }
            let slice = octave_str.as_slice()?;
            let as_utf8 =
                str::from_utf8(slice).map_err(|_| ParseOctaveError::Utf8Err)?;
            let result = NonZeroU8::from_str(as_utf8)?;
            Result::<_, ParseOctaveError>::Ok(result)
        };
        let optional_octave =
            many::<OctaveString, _, _>(satisfy(|b: u8| b.is_ascii_digit()))
                .map(parse_octave);
        (NoteLetter::parser(), optional_octave).map(|(letter, opt_octave)| {
            Result::<_, ParseNoteUnitError>::Ok(Note {
                letter,
                octave: opt_octave?,
            })
        })
    }
}

impl NoteUnit {
    pub fn parser<I: AstInput>()
    -> impl AstParser<I, Result<Self, ParseNoteUnitError>> + Clone {
        choice((
            NoteNumber::parser().map(|x| x.map(NoteUnit::Number)),
            Note::parser().map(|x| x.map(NoteUnit::WithOctave)),
        ))
    }
}
