use core::debug_assert_matches;

use fallible_iterator as fiter;
use fiter::FallibleIterator;

use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::timed_step_total_cycle_length;
use crate::ast::time::CycleTime;
use crate::gen_custom;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::query::QueryFrame;
use crate::interpreter::repeat::PlayMultiple;
use crate::interpreter::repeat::play_multiple_elements;
use crate::interpreter::value::ValueStack;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::structures::multiple;

#[derive(derive_more::From, Debug)]
pub enum ConcatFrame<'a, Arenas: PatternArenas> {
    CatOrSeq(CatOrSeqFrame<'a, Arenas>),
    Stack(StackFrame<'a, Arenas>),
    TimeCat(TimeCatFrame<'a, Arenas>),
    Arrange(ArrangeFrame<'a, Arenas>),
}

#[derive(Debug)]
pub struct CatOrSeqFrame<'a, Arenas: PatternArenas>(
    PlayMultiple<
        Index<PatternNode>,
        CatOrSeqProps<
            multiple::IterChecked<'a, PatternNode, PatternNode, Arenas, Arenas>,
        >,
    >,
);

#[derive(Debug)]
pub struct StackFrame<'a, Arenas>(
    fiter::Convert<
        multiple::IterChecked<'a, PatternNode, PatternNode, Arenas, Arenas>,
    >,
    PlayElemArgs<()>,
);

gen_custom! {
    #[derive(Debug)]
    struct CatOrSeqProps[Iter](
        opt_opt_index: Iter,
        _state: (),
    ) -> ElemProps<Index<PatternNode>>
    where struct []
    where iter [Iter: Iterator<Item = ArenaResult<Index<PatternNode>>>]
    {
        let Some(opt_index) = opt_opt_index else {
            return Ok(None);
        };

        Ok(Some(ElemProps {
            elem: opt_index?,
            sim_duration: CycleTime::ONE,
            played_duration: CycleTime::ONE,
        }))
    }
}

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

        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: move || CatOrSeqProps(multiple.checked_iter(arenas), ()),
            sim_duration: length,
            played_duration: length,
        });

        Ok(Self(play_multiple_elements(args_with_iterator, is_fast)?))
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

        let frame =
            Self(fiter::convert(multiple.checked_iter(arenas)), play_args);
        Ok(frame)
    }
}

#[derive(Debug)]
pub struct TimeCatFrame<'a, Arenas: PatternArenas>(
    PlayMultiple<
        Index<PatternNode>,
        TimeCatIter<
            'a,
            Arenas,
            multiple::IterChecked<'a, PatternNode, PatternNode, Arenas, Arenas>,
        >,
    >,
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
        .map(opt_index?, Clone::clone)?
        .pattern;
    let Pattern::TimedStep(timed_step) = cloned_pattern else {
        return Err(PatternInterpreterError::ExpectedTimedStep);
    };
    let TimedStep(elem_length, multiple) = timed_step.clone();
    let child =
        multiple.ok_or(PatternInterpreterError::ExpectedNonemptyTimedStep)?;

    PatternInterpreterResult::Ok(make_sim_elem(elem_length, child)?)
}

gen_custom! {
    struct TimeCatIter['a, Arenas, Iter](
        opt_opt_index: Iter,
        state: (CycleTime, CycleTime, &'a Arenas)
    ) -> ElemProps<Index<PatternNode>>
    where struct []
    where iter [
        Iter: Iterator<Item = ArenaResult<Index<PatternNode>>>,
        Arenas: PatternArenas,
    ]
    {
        let Some(opt_index) = opt_opt_index else {
            return Ok(None);
        };
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

        Ok(Some(make_sim_elem_in_arenas(arenas, opt_index, make_sim_elem)?))
    }
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

        debug_assert_matches!(
            timed_step_total_cycle_length(multiple.clone(), arenas),
            Ok(len) if len == total_cycle_length,
        );

        let multiple_length =
            CycleTime::checked_from_int(i32::from(multiple.length()))?;

        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: || {
                TimeCatIter(
                    multiple.checked_iter(arenas),
                    (total_cycle_length, multiple_length, arenas),
                )
            },
            sim_duration: multiple_length,
            played_duration: multiple_length,
        });

        Ok(Self(play_multiple_elements(args_with_iterator, true)?))
    }
}

#[derive(Debug)]
pub struct ArrangeFrame<'a, Arenas: PatternArenas>(
    PlayMultiple<
        Index<PatternNode>,
        ArrangeIter<
            'a,
            Arenas,
            multiple::IterChecked<'a, PatternNode, PatternNode, Arenas, Arenas>,
        >,
    >,
);

gen_custom! {
    struct ArrangeIter['a, Arenas, Iter](
        opt_opt_index: Iter,
        state: &'a Arenas,
    ) -> ElemProps<Index<PatternNode>>
    where struct []
    where iter [
        Iter: Iterator<Item = ArenaResult<Index<PatternNode>>>,
        Arenas: PatternArenas,
    ]
    {
        let arenas = *state;
        let Some(opt_index) = opt_opt_index else {
            return Ok(None);
        };

        let make_sim_elem = |elem_length, child_index| {
            Ok(ElemProps {
                elem: child_index,
                sim_duration: elem_length,
                played_duration: elem_length,
            })
        };

        Ok(Some(make_sim_elem_in_arenas(arenas, opt_index, make_sim_elem)?))
    }
}

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

        debug_assert_matches!(
            timed_step_total_cycle_length(multiple.clone(), arenas),
            Ok(length) if length == total_cycle_length,
        );

        let args_with_iterator = play_args.map(|()| ElemProps {
            elem: || ArrangeIter(multiple.checked_iter(arenas), arenas),
            sim_duration: total_cycle_length,
            played_duration: total_cycle_length,
        });

        Ok(Self(play_multiple_elements(args_with_iterator, false)?))
    }
}

impl<'a, Arenas: PatternArenas> ConcatFrame<'a, Arenas> {
    pub fn new_cat(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(
            multiple, arenas, play_args, false,
        )?))
    }

    pub fn new_seq(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(
            multiple, arenas, play_args, true,
        )?))
    }

    pub fn new_stack(
        multiple: &Multiple<PatternNode>,
        arenas: &'a Arenas,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::Stack(StackFrame::new(multiple, arenas, play_args)?))
    }

    pub fn new_time_cat(
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

    pub fn new_arrange(
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
        _: &mut impl ValueStack,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        Ok(self
            .0
            .next()?
            .map(QueryFrame::new)
            .map(Into::into))
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for StackFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        let args = self.1.clone();
        Ok(self
            .0
            .next()?
            .map(|index| QueryFrame::new(args.map(|()| index)).into()))
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for TimeCatFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        Ok(self
            .0
            .next()?
            .map(QueryFrame::new)
            .map(Into::into))
    }
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas>
    for ArrangeFrame<'a, Arenas>
{
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        _: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        Ok(self
            .0
            .next()?
            .map(QueryFrame::new)
            .map(Into::into))
    }
}
