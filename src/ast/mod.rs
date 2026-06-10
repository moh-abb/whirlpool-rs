pub mod pattern;
pub mod time;

pub use pattern::Pattern;
pub use pattern::TimedStep;
pub use pattern::note::Note;
pub use pattern::note::NoteFrequency;
pub use pattern::note::NoteLetter;
pub use pattern::note::NoteNumber;
pub use pattern::note::NoteUnit;
pub use time::CycleTime;
pub use time::interval::CycleInterval;

mod macros {
    macro_rules! compose_result {
        ($x: expr) => {
            match $x {
                Err(e) => return Err(e),
                Ok(x_inner) => x_inner,
            }
        };
    }

    pub(crate) use compose_result;
}
