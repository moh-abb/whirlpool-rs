use core::iter;
use core::ops::ControlFlow;

use crate::ast::CycleTime;
use crate::ast::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::elements::PlayMultiple;
use crate::interpreter::elements::play_multiple_elements;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::frame::query_frame;
use crate::interpreter::props::ElemProps;
use crate::interpreter::props::PlayElemArgs;
use crate::mem::ArenaResult;
use crate::mem::Index;
use crate::mem::Multiple;
use crate::mem::structures::multiple;
use crate::synth::scheduler::UnitScheduler;

pub enum ConcatFrame<'a, Arenas> {
    CatOrSeq(CatOrSeqFrame<'a, Arenas>),
    Stack(StackFrame<'a, Arenas>),
}

type CatOrSeqIter<'a, Arenas> = iter::Map<
    multiple::IterChecked<'a, PatternNode, PatternNode, Arenas>,
    fn(
        ArenaResult<Index<PatternNode>>,
    ) -> PatternInterpreterResult<ElemProps<Index<PatternNode>>>,
>;

pub struct CatOrSeqFrame<'a, Arenas>(
    PlayMultiple<Index<PatternNode>, CatOrSeqIter<'a, Arenas>>,
);

type StackIter<'a, Arenas> =
    multiple::IterChecked<'a, PatternNode, PatternNode, Arenas>;

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
            None => Ok(ControlFlow::Break(())),
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
            None => Ok(ControlFlow::Break(())),
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
