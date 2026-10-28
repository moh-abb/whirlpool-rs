use fallible_iterator as fiter;
use fallible_iterator::FallibleIterator;

use crate::ast::note::NoteUnit;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::CycleTime;
use crate::interpreter::elements::PlayMultiple;
use crate::interpreter::elements::play_multiple_elements;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::synth::scheduler::UnitScheduler;
use crate::synth::unit::SoundUnit;

#[derive(derive_more::From, Debug)]
pub enum LeafFrame {
    Note(NoteFrame),
    Silence(SilenceFrame),
}

#[derive(Debug)]
pub struct NoteFrame(
    PlayMultiple<
        NoteUnit,
        fiter::Once<ElemProps<NoteUnit>, PatternInterpreterError>,
    >,
);

impl NoteFrame {
    fn new(
        note: NoteUnit,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: move || {
                fiter::once(ElemProps {
                    elem: note,
                    sim_duration: CycleTime::ONE,
                    played_duration: CycleTime::ONE,
                })
            },
            sim_duration: CycleTime::ONE,
            played_duration: CycleTime::ONE,
        });
        Ok(Self(play_multiple_elements(args_with_iterator, false)?))
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas> for NoteFrame {
    fn step(
        &mut self,
        scheduler: &mut impl UnitScheduler,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        loop {
            let Some(args) = self.0.next()? else {
                return Ok(None);
            };
            // We could potentially cache the scheduled sound unit if this
            // affects performance.
            let unit_duration = args.multiplier.recip()?;
            let sound_unit = SoundUnit::new(args.elem, unit_duration);
            // Only play exact integer times, aligned to a single cycle
            let start = args.interval.start();
            if start != start.floor()? {
                continue;
            }
            let scaled_start = start
                .add(args.offset)?
                .div(args.multiplier)?;
            scheduler.add(sound_unit.clone(), scaled_start);
        }
    }
}

#[derive(Debug)]
pub struct SilenceFrame;

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas> for SilenceFrame {
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        Ok(None)
    }
}

impl LeafFrame {
    pub fn new_note(
        note: &NoteUnit,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::Note(NoteFrame::new(note.clone(), play_args)?))
    }

    pub fn new_silence() -> PatternInterpreterResult<Self> {
        Ok(Self::Silence(SilenceFrame))
    }
}
