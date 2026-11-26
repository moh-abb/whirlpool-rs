use crate::ast::expr::linked::ExprArenas;
use crate::ast::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::query::QueryMode;
use crate::interpreter::repeat::PlayMultiple;
use crate::interpreter::repeat::TimingSource;
use crate::interpreter::repeat::TimingStrategy;
use crate::interpreter::value::ValueStack;
use crate::synth::unit::SoundUnit;

#[derive(derive_more::From, Debug, Clone)]
pub enum LeafFrame {
    QueryUnit(QueryUnitFrame),
    PlayUnit(PlayUnitFrame),
    Silence(SilenceFrame),
}

#[derive(Debug, Clone)]
pub struct QueryUnitFrame(PlayMultiple<OneCycle>);

#[derive(Debug, Clone)]
pub struct PlayUnitFrame {
    unit: NoteUnit,
    play_args: PlayElemArgs<()>,
}

#[derive(Debug, Clone)]
pub struct SilenceFrame;

#[derive(Debug, Clone)]
struct OneCycle(Option<NoteUnit>);

impl TimingStrategy for OneCycle {
    type Item = NoteUnit;

    fn next_item(
        &mut self,
        _: &impl ExprArenas,
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
        _: &impl ExprArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        Ok(TimingSource::Const(ElemProps {
            elem: (),
            sim_duration: CycleTime::ONE,
            played_duration: CycleTime::ONE,
        }))
    }

    fn eval_item(
        &mut self,
        item: Self::Item,
        intersection: PlayElemArgs<()>,
        _query_mode: QueryMode,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(LeafFrame::from(PlayUnitFrame {
            unit: item,
            play_args: intersection,
        })
        .into())
    }
}

impl EvaluateFrame for QueryUnitFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl ExprArenas,
    ) -> InterpreterResult {
        self.0.step(value_stack, arenas)
    }
}

impl EvaluateFrame for PlayUnitFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        _: &impl ExprArenas,
    ) -> InterpreterResult {
        // We could potentially cache the scheduled sound unit if this
        // affects performance.
        let unit_duration = self.play_args.multiplier.recip()?;
        // Only play exact integer times, aligned to a single cycle
        let start = self.play_args.interval.start();
        if start != start.floor()? {
            return Ok(None);
        }
        let scaled_start = start
            .add(self.play_args.offset)?
            .div(self.play_args.multiplier)?;
        let sound_unit =
            SoundUnit::new(self.unit.clone(), scaled_start, unit_duration);
        value_stack.push(sound_unit.clone())?;
        Ok(None)
    }
}

impl EvaluateFrame for SilenceFrame {
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        _: &impl ExprArenas,
    ) -> InterpreterResult {
        Ok(None)
    }
}

impl LeafFrame {
    pub fn new_unit(
        note: NoteUnit,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::QueryUnit(QueryUnitFrame(PlayMultiple::new(
            play_args,
            query_mode,
            OneCycle(Some(note)),
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
        arenas: &impl ExprArenas,
    ) -> InterpreterResult {
        match self {
            Self::QueryUnit(frame) => frame.step(value_stack, arenas),
            Self::PlayUnit(frame) => frame.step(value_stack, arenas),
            Self::Silence(frame) => frame.step(value_stack, arenas),
        }
    }
}
