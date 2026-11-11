use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::CycleTime;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::query::QueryFrame;
use crate::interpreter::repeat::PlayMultiple;
use crate::interpreter::repeat::TimingSource;
use crate::interpreter::repeat::TimingStrategy;
use crate::interpreter::value::ValueStack;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::arena::iterator::ArenaIterator;
use crate::mem::structures::multiple;

#[derive(derive_more::From, Debug, Clone)]
pub enum ConcatFrame {
    CatOrSeq(CatOrSeqFrame),
    Stack(StackFrame),
    TimeCat(TimeCatFrame),
    Arrange(ArrangeFrame),
}

#[derive(Debug, Clone)]
pub struct CatOrSeqFrame(PlayMultiple<MultipleOfOneCycle>);

#[derive(Debug, Clone)]
pub struct StackFrame(
    multiple::Iter<PatternNode, PatternNode>,
    PlayElemArgs<()>,
);

#[derive(Debug, Clone)]
pub struct TimeCatFrame(PlayMultiple<TimeCatCycle>);

#[derive(Debug, Clone)]
pub struct ArrangeFrame(PlayMultiple<ArrangeCycle>);

/// Denotes that each element has one cycle simulated and played.
#[derive(Debug, Clone)]
struct MultipleOfOneCycle {
    iter: multiple::Iter<PatternNode, PatternNode>,
    total_cycle_length: CycleTime,
}

/// Denotes that given a timed step, the number of cycles is simulated, but
/// only played for one cycle. Used only with `Pattern::TimeCat`.
/// Needs the total cycle length as well, in order to scale down the played
/// intersection.
#[derive(Debug, Clone)]
struct TimeCatCycle {
    multiple_length: CycleTime,
    total_cycle_length: CycleTime,
    iter: multiple::Iter<PatternNode, PatternNode>,
}

/// Denotes that given a timed step, the number of cycles is both played and
/// simulated. Used only with `Pattern::Arrange`.
#[derive(derive_new::new, Debug, Clone)]
struct ArrangeCycle {
    total_cycle_length: CycleTime,
    iter: multiple::Iter<PatternNode, PatternNode>,
}

impl TimingStrategy for MultipleOfOneCycle {
    type Item = Index<PatternNode>;

    fn next_item(
        &mut self,
        arenas: &impl PatternArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>
    {
        let Some(elem) = self.iter.next_in_arena(arenas)? else {
            return Ok(None);
        };
        Ok(Some(TimingSource::Const(ElemProps {
            elem,
            sim_duration: CycleTime::ONE,
            played_duration: CycleTime::ONE,
        })))
    }

    fn total_props(
        &mut self,
        _: &impl PatternArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        Ok(TimingSource::Const(ElemProps {
            elem: (),
            sim_duration: self.total_cycle_length,
            played_duration: self.total_cycle_length,
        }))
    }

    fn eval_item(
        &mut self,
        intersection: PlayElemArgs<Self::Item>,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(QueryFrame::new(intersection).into())
    }
}

impl TimingStrategy for TimeCatCycle {
    type Item = Index<PatternNode>;

    fn next_item(
        &mut self,
        arenas: &impl PatternArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>
    {
        let Some(index) = self.iter.next_in_arena(arenas)? else {
            return Ok(None);
        };

        // If we have a [TimeCat], then we simulate over each one cycle
        // in the [Multiple] and then scale each element individually
        // by its proportion of the total;
        // e.g. TimeCat([1, "A"], [2, "B"], [3, "C"])
        // will have element lengths 1*3/6, 2*3/6, 3*3/6
        // (which adds to 3, the number of elements).
        let get_scaled_length = |elem_length: CycleTime| {
            elem_length
                .mul(self.multiple_length)?
                .div(self.total_cycle_length)
        };

        let make_sim_elem = |elem_length, child_index| {
            Result::<_, PatternInterpreterError>::Ok(ElemProps {
                elem: child_index,
                sim_duration: CycleTime::ONE,
                played_duration: get_scaled_length(elem_length)?,
            })
        };

        Ok(Some(TimingSource::Const(make_sim_elem_in_arenas(
            arenas,
            index,
            make_sim_elem,
        )?)))
    }

    fn total_props(
        &mut self,
        _: &impl PatternArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        Ok(TimingSource::Const(ElemProps {
            elem: (),
            sim_duration: self.multiple_length,
            played_duration: self.multiple_length,
        }))
    }

    fn eval_item(
        &mut self,
        intersection: PlayElemArgs<Self::Item>,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(QueryFrame::new(intersection).into())
    }
}

impl TimingStrategy for ArrangeCycle {
    type Item = Index<PatternNode>;

    fn next_item(
        &mut self,
        arenas: &impl PatternArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>
    {
        let Some(index) = self.iter.next_in_arena(arenas)? else {
            return Ok(None);
        };

        let make_sim_elem = |elem_length, child_index| {
            Ok(ElemProps {
                elem: child_index,
                sim_duration: elem_length,
                played_duration: elem_length,
            })
        };

        Ok(Some(TimingSource::Const(make_sim_elem_in_arenas(
            arenas,
            index,
            make_sim_elem,
        )?)))
    }

    fn total_props(
        &mut self,
        _: &impl PatternArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        Ok(TimingSource::Const(ElemProps {
            elem: (),
            sim_duration: self.total_cycle_length,
            played_duration: self.total_cycle_length,
        }))
    }

    fn eval_item(
        &mut self,
        intersection: PlayElemArgs<Self::Item>,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(QueryFrame::new(intersection).into())
    }
}

impl CatOrSeqFrame {
    fn new(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
        is_fast: bool,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        let length = CycleTime::checked_from_int(i32::from(multiple.length()))?;

        Ok(Self(PlayMultiple::new(
            play_args,
            MultipleOfOneCycle {
                iter: multiple.iter(),
                total_cycle_length: length,
            },
            is_fast,
        )))
    }
}

fn make_sim_elem_in_arenas<'a, Arenas, F>(
    arenas: &'a Arenas,
    index: Index<PatternNode>,
    make_sim_elem: F,
) -> PatternInterpreterResult<ElemProps<Index<PatternNode>>>
where
    Arenas: PatternArenas,
    F: FnOnce(
        CycleTime,
        Index<PatternNode>,
    ) -> PatternInterpreterResult<ElemProps<Index<PatternNode>>>,
{
    let cloned_pattern = arenas.map(index, Clone::clone)?.pattern;
    let Pattern::TimedStep(timed_step) = cloned_pattern else {
        return Err(PatternInterpreterError::ExpectedTimedStep);
    };
    let TimedStep(elem_length, multiple) = timed_step.clone();
    let child =
        multiple.ok_or(PatternInterpreterError::ExpectedNonemptyTimedStep)?;

    PatternInterpreterResult::Ok(make_sim_elem(elem_length, child)?)
}

impl ConcatFrame {
    pub fn new_cat(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(multiple, play_args, false)?))
    }

    pub fn new_seq(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(multiple, play_args, true)?))
    }

    pub fn new_stack(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        Ok(Self::Stack(StackFrame(multiple.iter(), play_args)))
    }

    pub fn new_time_cat(
        multiple: &Multiple<PatternNode>,
        total_cycle_length: CycleTime,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        let multiple_length =
            CycleTime::checked_from_int(i32::from(multiple.length()))?;

        Ok(Self::TimeCat(TimeCatFrame(PlayMultiple::new(
            play_args,
            TimeCatCycle {
                multiple_length,
                total_cycle_length,
                iter: multiple.iter(),
            },
            true,
        ))))
    }

    pub fn new_arrange(
        multiple: &Multiple<PatternNode>,
        total_cycle_length: CycleTime,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        Ok(Self::Arrange(ArrangeFrame(PlayMultiple::new(
            play_args,
            ArrangeCycle { total_cycle_length, iter: multiple.iter() },
            false,
        ))))
    }
}

impl EvaluateFrame for StackFrame {
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        let args = self.1.clone();
        Ok(self
            .0
            .next_in_arena(arenas)?
            .map(|index| QueryFrame::new(args.map(|()| index)).into()))
    }
}

impl EvaluateFrame for ConcatFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        // TODO: Delegate this step.
        match self {
            ConcatFrame::Stack(frame) => frame.step(value_stack, arenas),
            ConcatFrame::CatOrSeq(frame) => frame.0.step(value_stack, arenas),
            ConcatFrame::TimeCat(frame) => frame.0.step(value_stack, arenas),
            ConcatFrame::Arrange(frame) => frame.0.step(value_stack, arenas),
        }
    }
}
