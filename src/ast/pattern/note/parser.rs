use core::convert::Infallible;
use core::num::NonZeroU8;
use core::num::ParseIntError;
use core::str::FromStr;
use core::str::Utf8Error;

use chumsky::IterParser;
use chumsky::Parser;
use chumsky::primitive::any;
use chumsky::primitive::choice;
use chumsky::primitive::just;

use crate::ast::CycleTime;
use crate::ast::Note;
use crate::ast::NoteLetter;
use crate::ast::NoteNumber;
use crate::ast::NoteUnit;
use crate::ast::parser::AstParser;
use crate::ast::parser::Input;
use crate::ast::parser::Parseable;
use crate::ast::parser::array::ParserVec;
use crate::ast::parser::array::ParserVecError;
use crate::ast::time::parser::ParseCycleTimeError;

#[derive(derive_more::From, Debug)]
pub enum ParseNoteUnitError {
    CycleTimeErr(ParseCycleTimeError),
    ParseOctaveErr(ParseOctaveError),
    InfallibleErr(Infallible),
}

#[derive(derive_more::From, Debug)]
pub enum ParseOctaveError {
    FromIntErr(ParseIntError),
    Utf8Err(Utf8Error),
    StringBufferErr(ParserVecError),
}

const OCTAVE_STRING_CAPACITY: usize = 8;
type OctaveString = ParserVec<u8, OCTAVE_STRING_CAPACITY>;

impl<'src> Parseable<'src, ()> for NoteNumber {
    type Error = ParseNoteUnitError;

    fn parser<I: Input<'src>>(
        _args: (),
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>> {
        CycleTime::parser(()).map(|x| {
            x.map(NoteNumber)
                .map_err(ParseNoteUnitError::CycleTimeErr)
        })
    }
}

impl<'src> Parseable<'src, ()> for NoteLetter {
    type Error = Infallible;

    fn parser<I: Input<'src>>(
        _args: (),
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>> {
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
            |byte: u8| just(byte).or(just(byte.to_ascii_uppercase()));
        let aug_note_parsers = aug_notes.map(|(note_byte, aug_byte, res)| {
            ignore_case(note_byte)
                .then(ignore_case(aug_byte))
                .to(res)
        });
        let note_map_parsers = notes.map(|(note_char, res)| {
            just(note_char)
                .or(just(note_char.to_ascii_uppercase()))
                .to(res)
        });
        choice(aug_note_parsers)
            .or(choice(note_map_parsers))
            .map(Ok)
    }
}

impl<'src> Parseable<'src, ()> for Note {
    type Error = ParseNoteUnitError;

    fn parser<I: Input<'src>>(
        _args: (),
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>> {
        let octave = any()
            .filter(u8::is_ascii_digit)
            .repeated()
            .at_least(1)
            .collect::<OctaveString>()
            .map(|octave_str| {
                let slice = octave_str.as_slice()?;
                let as_utf8 = str::from_utf8(slice)?;
                let result = NonZeroU8::from_str(as_utf8)?;
                Result::<_, ParseOctaveError>::Ok(result)
            });
        let optional_octave = octave
            .or_not()
            .map(|opt_octave| opt_octave.unwrap_or(Ok(Note::DEFAULT_OCTAVE)));
        NoteLetter::parser(())
            .then(optional_octave)
            .map(|(opt_letter, opt_octave)| {
                Result::<_, ParseNoteUnitError>::Ok(Note {
                    letter: opt_letter?,
                    octave: opt_octave?,
                })
            })
    }
}

impl<'src> Parseable<'src, ()> for NoteUnit {
    type Error = ParseNoteUnitError;

    fn parser<I: Input<'src>>(
        _args: (),
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>> {
        choice((
            NoteNumber::parser(()).map(|x| x.map(NoteUnit::Number)),
            Note::parser(()).map(|x| x.map(NoteUnit::WithOctave)),
        ))
    }
}
