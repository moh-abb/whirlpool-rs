use core::iter;
use core::ops::ControlFlow;

use crate::ast::CycleTime;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::elements::PlayMultiple;
use crate::interpreter::elements::play_multiple_elements;
use crate::interpreter::elements::timed_step_total_cycle_length;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::frame::query_frame;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::Arena;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::structures::multiple;
use crate::synth::scheduler::UnitScheduler;

#[derive(derive_more::From, Debug)]
pub enum ConcatFrame<'a, Arenas: PatternArenas> {
    CatOrSeq(CatOrSeqFrame<'a, Arenas>),
    Stack(StackFrame<'a, Arenas>),
    TimeCat(TimeCatFrame<'a, Arenas>),
    Arrange(ArrangeFrame<'a, Arenas>),
}

type CatOrSeqIter<'a, Arenas> = iter::Map<
    multiple::IterChecked<'a, PatternNode, PatternNode, Arenas>,
    fn(
        ArenaResult<Index<PatternNode>>,
    ) -> PatternInterpreterResult<ElemProps<Index<PatternNode>>>,
>;

#[derive(Debug)]
pub struct CatOrSeqFrame<'a, Arenas: PatternArenas>(
    PlayMultiple<Index<PatternNode>, CatOrSeqIter<'a, Arenas>>,
);

type StackIter<'a, Arenas> =
    multiple::IterChecked<'a, PatternNode, PatternNode, Arenas>;

#[derive(Debug)]
pub struct StackFrame<'a, Arenas>(PlayElemArgs<()>, StackIter<'a, Arenas>);

impl<'a, Arenas: PatternArenas> CatOrSeqFrame<'a, Arenas> {
    fn new(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
        is_fast: bool,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        let length = CycleTime::checked_from_int(i32::from(multiple.length()))?;

        let opt_index_to_props: fn(_) -> _ = |opt_index| {
            PatternInterpreterResult::Ok(ElemProps {
                elem: opt_index?,
                sim_duration: CycleTime::ONE,
                played_duration: CycleTime::ONE,
            })
        };

        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: move || {
                multiple
                    .checked_iter(arenas)
                    .map(opt_index_to_props)
            },
            sim_duration: length,
            played_duration: length,
        });
        let frame = Self(play_multiple_elements(args_with_iterator, is_fast)?);
        Ok(frame)
    }
}

impl<'a, Arenas: PatternArenas> StackFrame<'a, Arenas> {
    fn new(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        let frame = Self(play_args, multiple.checked_iter(arenas));
        Ok(frame)
    }
}

type TimeCatIter<'a, Arenas> = iter::Scan<
    multiple::IterChecked<'a, PatternNode, PatternNode, Arenas>,
    (CycleTime, CycleTime, &'a Arenas),
    fn(
        &mut (CycleTime, CycleTime, &'a Arenas),
        ArenaResult<Index<PatternNode>>,
    ) -> Option<PatternInterpreterResult<ElemProps<Index<PatternNode>>>>,
>;

#[derive(Debug)]
pub struct TimeCatFrame<'a, Arenas: PatternArenas>(
    PlayMultiple<Index<PatternNode>, TimeCatIter<'a, Arenas>>,
);

fn make_sim_elem_in_arenas<'a, Arenas, F>(
    arenas: &'a Arenas,
    opt_index: ArenaResult<Index<PatternNode>>,
    make_sim_elem: F,
) -> PatternInterpreterResult<ElemProps<Index<PatternNode>>>
where
    Arenas: PatternArenas,
    F: FnOnce(
        CycleTime,
        Index<PatternNode>,
    ) -> PatternInterpreterResult<ElemProps<Index<PatternNode>>>,
{
    let cloned_pattern = arenas
        .get_pattern_arena()
        .map(opt_index?, Clone::clone)?
        .pattern;
    let Pattern::TimedStep(timed_step) = cloned_pattern else {
        return Err(PatternInterpreterError::ExpectedTimedStep);
    };
    let TimedStep(elem_length, multiple) = timed_step.clone();
    if multiple.length() > 1 {
        return Err(PatternInterpreterError::OverOneChildInTimedStep);
    }
    let child = multiple
        .start()
        .ok_or(PatternInterpreterError::ExpectedNonemptyTimedStep)?;

    PatternInterpreterResult::Ok(make_sim_elem(elem_length, child)?)
}

impl<'a, Arenas: PatternArenas> TimeCatFrame<'a, Arenas> {
    fn new(
        multiple: &Multiple<PatternNode>,
        &total_cycle_length: &CycleTime,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        debug_assert_eq!(
            Ok(total_cycle_length),
            timed_step_total_cycle_length(&multiple, arenas),
        );

        let multiple_length =
            CycleTime::checked_from_int(i32::from(multiple.length()))?;

        let scan_func: fn(&mut _, _) -> _ =
            |state: &mut (CycleTime, CycleTime, &'a Arenas), opt_index| {
                let &(total_length, multiple_length, arenas) = &*state;

                // If we have a [TimeCat], then we simulate over each one cycle
                // in the [Multiple] and then scale each element individually
                // by its proportion of the total;
                // e.g. TimeCat([1, "A"], [2, "B"], [3, "C"])
                // will have element lengths 1*3/6, 2*3/6, 3*3/6
                // (which adds to 3, the number of elements).
                let get_scaled_length = |elem_length: CycleTime| {
                    elem_length
                        .mul(multiple_length)?
                        .div(total_length)
                };

                let make_sim_elem = |elem_length, child_index| {
                    Result::<_, PatternInterpreterError>::Ok(ElemProps {
                        elem: child_index,
                        sim_duration: CycleTime::ONE,
                        played_duration: get_scaled_length(elem_length)?,
                    })
                };

                Some(make_sim_elem_in_arenas(arenas, opt_index, make_sim_elem))
            };

        let get_elements = || {
            multiple
                .checked_iter(arenas)
                .scan((total_cycle_length, multiple_length, arenas), scan_func)
        };

        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: get_elements,
            sim_duration: multiple_length,
            played_duration: total_cycle_length,
        });
        let frame = Self(play_multiple_elements(args_with_iterator, true)?);
        Ok(frame)
    }
}

type ArrangeIter<'a, Arenas> = iter::Scan<
    multiple::IterChecked<'a, PatternNode, PatternNode, Arenas>,
    &'a Arenas,
    fn(
        &mut &'a Arenas,
        ArenaResult<Index<PatternNode>>,
    ) -> Option<PatternInterpreterResult<ElemProps<Index<PatternNode>>>>,
>;

#[derive(Debug)]
pub struct ArrangeFrame<'a, Arenas: PatternArenas>(
    PlayMultiple<Index<PatternNode>, ArrangeIter<'a, Arenas>>,
);

impl<'a, Arenas: PatternArenas> ArrangeFrame<'a, Arenas> {
    fn new(
        multiple: &Multiple<PatternNode>,
        &total_cycle_length: &CycleTime,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        debug_assert_eq!(
            Ok(total_cycle_length),
            timed_step_total_cycle_length(&multiple, arenas),
        );

        let scan_func: fn(&mut _, _) -> _ =
            |state: &mut &'a Arenas, opt_index| {
                let &arenas = &*state;

                let make_sim_elem = |elem_length, child_index| {
                    Ok(ElemProps {
                        elem: child_index,
                        sim_duration: elem_length,
                        played_duration: elem_length,
                    })
                };

                Some(make_sim_elem_in_arenas(arenas, opt_index, make_sim_elem))
            };

        let get_elements = || {
            multiple
                .checked_iter(arenas)
                .scan(arenas, scan_func)
        };

        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: get_elements,
            sim_duration: total_cycle_length,
            played_duration: total_cycle_length,
        });
        let frame = Self(play_multiple_elements(args_with_iterator, true)?);
        Ok(frame)
    }
}

impl<'a, Arenas: PatternArenas> ConcatFrame<'a, Arenas> {
    pub fn cat_frame(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(
            multiple, arenas, play_args, false,
        )?))
    }

    pub fn seq_frame(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(
            multiple, arenas, play_args, true,
        )?))
    }

    pub fn stack_frame(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::Stack(StackFrame::new(multiple, arenas, play_args)?))
    }

    pub fn time_cat_frame(
        multiple: &Multiple<PatternNode>,
        total_cycle_length: &CycleTime,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::TimeCat(TimeCatFrame::new(
            multiple,
            total_cycle_length,
            arenas,
            play_args,
        )?))
    }

    pub fn arrange_frame(
        multiple: &Multiple<PatternNode>,
        total_cycle_length: &CycleTime,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::Arrange(ArrangeFrame::new(
            multiple,
            total_cycle_length,
            arenas,
            play_args,
        )?))
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for CatOrSeqFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        match self.0.next() {
            None => Ok(ControlFlow::Break(None)),
            Some(Err(err)) => Err(err),
            Some(Ok(args)) => {
                Ok(ControlFlow::Continue(Some(query_frame(args))))
            }
        }
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for StackFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        let checked_iter = &mut self.1;
        match checked_iter.next() {
            None => Ok(ControlFlow::Break(None)),
            Some(Err(err)) => Err(err.into()),
            Some(Ok(index)) => {
                let args = self.0.clone();
                Ok(ControlFlow::Continue(Some(query_frame(
                    args.map(|()| index),
                ))))
            }
        }
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for TimeCatFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        match self.0.next() {
            None => Ok(ControlFlow::Break(None)),
            Some(Err(err)) => Err(err),
            Some(Ok(args)) => {
                Ok(ControlFlow::Continue(Some(query_frame(args))))
            }
        }
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for ArrangeFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        match self.0.next() {
            None => Ok(ControlFlow::Break(None)),
            Some(Err(err)) => Err(err),
            Some(Ok(args)) => {
                Ok(ControlFlow::Continue(Some(query_frame(args))))
            }
        }
    }
}
