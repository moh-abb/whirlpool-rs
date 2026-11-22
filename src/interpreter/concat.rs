use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::timed_step_total_cycle_length;
use crate::ast::time::CycleTime;
use crate::impl_frame;
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
    TimeCatOrArrange(PlayTimeCatOrArrangeFrame),
    GetTotalCycles(GetTotalCyclesFrame),
    GetTimedStep(GetTimedStep),
}

#[derive(Debug, Clone)]
pub struct CatOrSeqFrame(PlayMultiple<MultipleOfOneCycle>);

#[derive(Debug, Clone)]
pub struct StackFrame(
    multiple::Iter<PatternNode, PatternNode>,
    PlayElemArgs<()>,
);

/// Given a list of timed steps, finds their total length and places them onto
/// the value stack.
#[derive(Debug, Clone)]
pub struct GetTotalCyclesFrame {
    multiple: Multiple<PatternNode>,
}

/// Takes an index and finds the corresponding timed step, pushing it onto the
/// value stack.
#[derive(Debug, Clone)]
pub struct GetTimedStep(Index<PatternNode>);

/// Denotes that each element has one cycle simulated and played.
#[derive(Debug, Clone)]
struct MultipleOfOneCycle {
    iter: multiple::Iter<PatternNode, PatternNode>,
    total_cycle_length: CycleTime,
}

/// For Arrange: Denotes that given a timed step, the number of cycles is both
/// played and simulated. Used only with `Pattern::Arrange`.
///
/// For TimeCat:
/// Denotes that given a timed step, the number of cycles is simulated, but
/// only played for one cycle. Used only with `Pattern::TimeCat`.
/// Needs the total cycle length as well, in order to scale down the played
/// intersection.
#[derive(Debug, Clone)]
struct TimeCatOrArrangeCycle {
    multiple_length: CycleTime,
    total_cycle_length: CycleTime,
    is_time_cat: bool,
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

    fn pop_elem_props(
        &mut self,
        _: &mut impl ValueStack,
    ) -> PatternInterpreterResult<ElemProps<Self::Item>> {
        // We didn't get the elem props by a frame so no need to pop from the
        // value stack.
        Err(PatternInterpreterError::UnexpectedPopElemProps)
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

impl TimingStrategy for TimeCatOrArrangeCycle {
    type Item = Index<PatternNode>;

    fn next_item(
        &mut self,
        arenas: &impl PatternArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>
    {
        let Some(index) = self.iter.next_in_arena(arenas)? else {
            return Ok(None);
        };

        Ok(Some(TimingSource::ViaFrame(
            ConcatFrame::GetTimedStep(GetTimedStep(index)).into(),
        )))
    }

    fn pop_elem_props(
        &mut self,
        stack: &mut impl ValueStack,
    ) -> PatternInterpreterResult<ElemProps<Self::Item>> {
        let TimedStep(elem_length, opt_child_index) = stack.pop()?;
        let child_index = opt_child_index
            .ok_or(PatternInterpreterError::ExpectedNonemptyTimedStep)?;

        let elem_props = if self.is_time_cat {
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

            ElemProps {
                elem: child_index,
                sim_duration: CycleTime::ONE,
                played_duration: get_scaled_length(elem_length)?,
            }
        } else {
            ElemProps {
                elem: child_index,
                sim_duration: elem_length,
                played_duration: elem_length,
            }
        };

        Ok(elem_props)
    }

    fn total_props(
        &mut self,
        _: &impl PatternArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        let elem_props = if self.is_time_cat {
            ElemProps {
                elem: (),
                sim_duration: self.multiple_length,
                played_duration: self.multiple_length,
            }
        } else {
            ElemProps {
                elem: (),
                sim_duration: self.total_cycle_length,
                played_duration: self.total_cycle_length,
            }
        };

        Ok(TimingSource::Const(elem_props))
    }

    fn eval_item(
        &mut self,
        intersection: PlayElemArgs<Self::Item>,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(QueryFrame::new(intersection).into())
    }
}

impl EvaluateFrame for GetTotalCyclesFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        let total_length =
            timed_step_total_cycle_length(self.multiple.clone(), arenas)?;
        value_stack.push(total_length)?;
        Ok(None)
    }
}

impl EvaluateFrame for GetTimedStep {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        let pattern = arenas
            .map(self.0.clone(), PatternNode::clone)?
            .pattern;
        let Pattern::TimedStep(timed_step) = pattern else {
            return Err(PatternInterpreterError::ExpectedTimedStep);
        };
        value_stack.push(timed_step)?;
        Ok(None)
    }
}

impl_frame!(
    /// Given a list of timed steps, uses another frame to find the total length
    /// and then plays the repetitions.
    #[derive(derive_more::Debug, Clone)]
    pub struct PlayTimeCatOrArrangeFrame {
        state: PlayTimeCatOrArrangeState,
    },
    [] = [] + [],
    /// The state for playing `TimeCat` or `Arrange`.
    #[derive(derive_more::Debug, Clone)]
    enum PlayTimeCatOrArrangeState {},
    [] = [],
    // STATES
    [self, value_stack, arenas],
    /// About to obtain the total number of cycles and begin the elements.
    fn start(
        play_args: PlayElemArgs<()>,
        multiple: Multiple<PatternNode>,
        is_time_cat: bool,
    ) {
        (
            pending_total_cycles {
                play_args,
                multiple: multiple.clone(),
                is_time_cat,
            },
            Some(
                ConcatFrame::GetTotalCycles(GetTotalCyclesFrame { multiple })
                    .into(),
            ),
        )
    },
    /// Pending the total number of cycles to be calculated from another frame.
    fn pending_total_cycles(
        play_args: PlayElemArgs<()>,
        multiple: Multiple<PatternNode>,
        is_time_cat: bool,
    ) {
        let total_cycle_length = value_stack.pop()?;
        let multiple_length =
            CycleTime::checked_from_int(i32::from(multiple.length()))?;
        (
            in_play_multiple {
                play_multiple: PlayMultiple::new(
                    play_args.clone(),
                    TimeCatOrArrangeCycle {
                        multiple_length,
                        total_cycle_length,
                        is_time_cat,
                        iter: multiple.iter(),
                    },
                    is_time_cat,
                ),
            },
            None,
        )
    },
    /// Currently processing elements using [PlayMultiple].
    fn in_play_multiple(play_multiple: PlayMultiple<TimeCatOrArrangeCycle>) {
        let mut mut_play_multiple = play_multiple;
        let opt_next_frame = mut_play_multiple.step(value_stack, arenas)?;
        if let Some(frame) = opt_next_frame {
            (in_play_multiple { play_multiple: mut_play_multiple }, Some(frame))
        } else {
            (finish, None)
        }
    }
);

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

    fn new_time_cat_or_arrange(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
        is_time_cat: bool,
    ) -> PatternInterpreterResult<Self> {
        if multiple.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        Ok(Self::TimeCatOrArrange(PlayTimeCatOrArrangeFrame {
            state: PlayTimeCatOrArrangeState::start {
                multiple: multiple.clone(),
                play_args,
                is_time_cat,
            },
        }))
    }

    pub fn new_time_cat(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Self::new_time_cat_or_arrange(multiple, play_args, true)
    }

    pub fn new_arrange(
        multiple: &Multiple<PatternNode>,
        play_args: PlayElemArgs<()>,
    ) -> PatternInterpreterResult<Self> {
        Self::new_time_cat_or_arrange(multiple, play_args, false)
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
            ConcatFrame::GetTimedStep(frame) => frame.step(value_stack, arenas),
            ConcatFrame::TimeCatOrArrange(frame) => {
                frame.step(value_stack, arenas)
            }
            ConcatFrame::GetTotalCycles(frame) => {
                frame.step(value_stack, arenas)
            }
        }
    }
}
