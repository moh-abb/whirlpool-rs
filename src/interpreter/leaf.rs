use core::iter;
use core::ops::ControlFlow;

use crate::ast::CycleTime;
use crate::ast::NoteUnit;
use crate::interpreter::elements::PlayMultiple;
use crate::interpreter::elements::play_multiple_elements;
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
        iter::Once<PatternInterpreterResult<ElemProps<NoteUnit>>>,
    >,
);

impl NoteFrame {
    fn new(
        note: NoteUnit,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: move || {
                iter::once(PatternInterpreterResult::Ok(ElemProps {
                    elem: note,
                    sim_duration: CycleTime::ONE,
                    played_duration: CycleTime::ONE,
                }))
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
        match self.0.next() {
            None => Ok(ControlFlow::Break(())),
            Some(Err(err)) => Err(err),
            Some(Ok(args)) => {
                // We could potentially cache the scheduled sound unit if this
                // affects performance.
                let unit_duration = args.multiplier.recip()?;
                let sound_unit = SoundUnit::new(args.elem, unit_duration);
                // Only play exact integer times, aligned to a single cycle
                let start = args.interval.start();
                if start != start.floor()? {
                    return Ok(ControlFlow::Continue(None));
                }
                let scaled_start = start
                    .add(args.offset)?
                    .div(args.multiplier)?;
                scheduler.add(sound_unit.clone(), scaled_start);
                Ok(ControlFlow::Continue(None))
            }
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
        Ok(ControlFlow::Break(()))
    }
}

impl LeafFrame {
    pub fn note_frame(
        note: &NoteUnit,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::Note(NoteFrame::new(note.clone(), play_args)?))
    }

    pub fn silence() -> PatternInterpreterResult<Self> {
        Ok(Self::Silence(SilenceFrame))
    }
}
