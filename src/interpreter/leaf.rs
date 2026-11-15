use crate::ast::note::NoteUnit;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::CycleTime;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::repeat::PlayMultiple;
use crate::interpreter::repeat::TimingSource;
use crate::interpreter::repeat::TimingStrategy;
use crate::interpreter::value::ValueStack;
use crate::synth::unit::SoundUnit;

#[derive(derive_more::From, Debug, Clone)]
pub enum LeafFrame {
    QueryNote(QueryNoteFrame),
    PlayNote(PlayNoteFrame),
    Silence(SilenceFrame),
}

#[derive(Debug, Clone)]
pub struct QueryNoteFrame(PlayMultiple<OneCycle>);

#[derive(Debug, Clone)]
pub struct PlayNoteFrame(PlayElemArgs<NoteUnit>);

#[derive(Debug, Clone)]
pub struct SilenceFrame;

#[derive(Debug, Clone)]
struct OneCycle(Option<NoteUnit>);

impl TimingStrategy for OneCycle {
    type Item = NoteUnit;

    fn next_item(
        &mut self,
        _: &impl PatternArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>
    {
        Ok(self.0.take().map(|elem| {
            TimingSource::Const(ElemProps {
                elem,
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            })
        }))
    }

    fn pop_elem_props(
        &mut self,
        _: &mut impl ValueStack,
    ) -> PatternInterpreterResult<ElemProps<Self::Item>> {
        // We returned a constant value from `next_item`, so shouldn't be
        // popping anything from the stack.
        Err(PatternInterpreterError::UnexpectedPopElemProps)
    }

    fn total_props(
        &mut self,
        _: &impl PatternArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        Ok(TimingSource::Const(ElemProps {
            elem: (),
            sim_duration: CycleTime::ONE,
            played_duration: CycleTime::ONE,
        }))
    }

    fn eval_item(
        &mut self,
        intersection: PlayElemArgs<Self::Item>,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(LeafFrame::from(PlayNoteFrame(intersection)).into())
    }
}

impl EvaluateFrame for QueryNoteFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        self.0.step(value_stack, arenas)
    }
}

impl EvaluateFrame for PlayNoteFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        _: &impl PatternArenas,
    ) -> InterpreterResult {
        let args = self.0.clone();
        // We could potentially cache the scheduled sound unit if this
        // affects performance.
        let unit_duration = args.multiplier.recip()?;
        // Only play exact integer times, aligned to a single cycle
        let start = args.interval.start();
        if start != start.floor()? {
            return Ok(None);
        }
        let scaled_start = start
            .add(args.offset)?
            .div(args.multiplier)?;
        let sound_unit = SoundUnit::new(args.elem, scaled_start, unit_duration);
        value_stack.push(sound_unit.clone())?;
        Ok(None)
    }
}

impl EvaluateFrame for SilenceFrame {
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        _: &impl PatternArenas,
    ) -> InterpreterResult {
        Ok(None)
    }
}

impl LeafFrame {
    pub fn new_note(
        note: &NoteUnit,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::QueryNote(QueryNoteFrame(PlayMultiple::new(
            play_args,
            OneCycle(Some(note.clone())),
            false,
        ))))
    }

    pub fn new_silence() -> PatternInterpreterResult<Self> {
        Ok(Self::Silence(SilenceFrame))
    }
}

impl EvaluateFrame for LeafFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        match self {
            LeafFrame::QueryNote(frame) => frame.step(value_stack, arenas),
            LeafFrame::PlayNote(frame) => frame.step(value_stack, arenas),
            LeafFrame::Silence(frame) => frame.step(value_stack, arenas),
        }
    }
}
