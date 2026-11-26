use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::call_arg::CallArg;
use crate::ast::expr::call_arg::CallArgs;
use crate::ast::expr::call_arg::CallArgsIter;
use crate::ast::expr::linked::ExprArenas;
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
use crate::interpreter::query::QueryMode;
use crate::interpreter::repeat::PlayMultiple;
use crate::interpreter::repeat::TimingSource;
use crate::interpreter::repeat::TimingStrategy;
use crate::interpreter::value::TimedStep;
use crate::interpreter::value::ValueStack;
use crate::mem::Index;
use crate::mem::arena::iterator::ArenaIterator;

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
pub struct StackFrame {
    call_args_iter: CallArgsIter<ExprNode>,
    play_args: PlayElemArgs<()>,
    query_mode: QueryMode,
}

/// Denotes that each element has one cycle simulated and played.
#[derive(Debug, Clone)]
struct MultipleOfOneCycle {
    iter: CallArgsIter<ExprNode>,
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
    iter: CallArgsIter<ExprNode>,
}

impl TimingStrategy for MultipleOfOneCycle {
    type Item = CallArg;

    fn next_item(
        &mut self,
        arenas: &impl ExprArenas,
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
        _: &impl ExprArenas,
    ) -> PatternInterpreterResult<TimingSource<ElemProps<()>>> {
        Ok(TimingSource::Const(ElemProps {
            elem: (),
            sim_duration: self.total_cycle_length,
            played_duration: self.total_cycle_length,
        }))
    }

    fn eval_item(
        &mut self,
        item: Self::Item,
        intersection: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(QueryFrame::new_non_head(item, intersection, query_mode)?.into())
    }
}

impl TimingStrategy for TimeCatOrArrangeCycle {
    type Item = Index<ExprNode>;

    fn next_item(
        &mut self,
        arenas: &impl ExprArenas,
    ) -> PatternInterpreterResult<Option<TimingSource<ElemProps<Self::Item>>>>
    {
        let Some(index) = self.iter.next_in_arena(arenas)? else {
            return Ok(None);
        };

        Ok(Some(TimingSource::ViaFrame(
            ConcatFrame::GetTimedStep(GetTimedStep::new(index, true)).into(),
        )))
    }

    fn pop_elem_props(
        &mut self,
        stack: &mut impl ValueStack,
    ) -> PatternInterpreterResult<ElemProps<Self::Item>> {
        let TimedStep(elem_length, child_index) = stack.pop()?;
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
        _: &impl ExprArenas,
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
        item: Self::Item,
        intersection: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<InterpreterFrame> {
        Ok(QueryFrame::new_non_head(
            CallArg::Index(item),
            intersection,
            query_mode,
        )?
        .into())
    }
}

impl_frame!(
    /// Gets the total number of cycles of a list of arguments, assuming these
    /// arguments are stored as multiple arrays, each with two elements, and
    /// the first element being a number (the cycle length of that element).
    /// Places the total length onto the stack on completion.
    #[derive(Debug, Clone)]
    pub struct GetTotalCyclesFrame {
        state: GetTotalCyclesState,
    },
    [] = [] + [],
    /// The state for getting the total number of cycles.
    #[derive(Debug, Clone)]
    enum GetTotalCyclesState {},
    [] = [],
    [self, value_stack, arenas],
    fn start(call_args: CallArgs) {
        (middle { outer: call_args.iter(), acc: CycleTime::ZERO }, None)
    },
    fn middle(outer: CallArgsIter<ExprNode>, acc: CycleTime) {
        let mut mut_outer = outer;
        match mut_outer.next_in_arena(arenas)? {
            Some(next) => {
                (pending_timed_step { outer: mut_outer, elem: next, acc }, None)
            }
            None => (push_value { acc }, None),
        }
    },
    /// Pending the timed step to be calculated using [GetTimedStep].
    fn pending_timed_step(
        outer: CallArgsIter<ExprNode>,
        elem: CallArg,
        acc: CycleTime,
    ) {
        (
            pop_timed_step { outer, acc },
            Some(
                ConcatFrame::GetTimedStep(GetTimedStep::new(elem, false))
                    .into(),
            ),
        )
    },
    /// Pop the cycle time that was pushed by [GetTimedStep].
    fn pop_timed_step(outer: CallArgsIter<ExprNode>, acc: CycleTime) {
        let cur = value_stack.pop::<CycleTime>()?;
        (middle { outer, acc: acc.add(cur)? }, None)
    },
    /// At the end, push the total cycle time.
    fn push_value(acc: CycleTime) {
        value_stack.push(acc)?;
        (finish, None)
    }
);

impl_frame!(
    /// Gets the timed step (i.e., an array with two elements: a cycle time,
    /// and an element to play) mapping to a specific expression's index.
    #[derive(derive_new::new, Debug, Clone)]
    pub struct GetTimedStep {
        #[new(value = "GetTimedStepState::start {}")]
        state: GetTimedStepState,
        /// The index or identifier that should point to a timed step.
        call_arg: CallArg,
        /// Should we push a full timed step (true)
        /// or just the timed step's length (false)?
        get_elem: bool,
    },
    [] = [] + [],
    /// The state for getting the current timed step.
    #[derive(Debug, Clone)]
    enum GetTimedStepState {},
    [] = [],
    [self, value_stack, arenas],
    /// Follow the index until we get an actual array.
    fn start() {
        // `elem` should point to an array with two elements: the first being
        // the timed step's cycle time, and the second being the element.
        // TODO: handle let-bindings, this will translate to a loop of finding
        // where the `let` maps to and getting that index as the next value of
        // `elem`.
        let expr = match self.call_arg.clone() {
            CallArg::Ident(_ident) => todo!(),
            CallArg::Index(index) => {
                arenas
                    .map(index.clone(), Clone::clone)?
                    .0
                    .item
            }
        };
        let Expr::Array { args } = expr else {
            return Err(PatternInterpreterError::ExpectedTimedStep);
        };
        let mut inner_args = args.iter();
        let Some(first_arg) = inner_args.next_in_arena(arenas)? else {
            return Err(PatternInterpreterError::ExpectedNonemptyTimedStep);
        };
        let Some(second_arg) = inner_args.next_in_arena(arenas)? else {
            return Err(
                PatternInterpreterError::ExpectedTwoElementsInTimedStep,
            );
        };
        (resolve_first { first_arg, second_arg }, None)
    },
    fn resolve_first(first_arg: Index<ExprNode>, second_arg: Index<ExprNode>) {
        // Get the first element.
        // TODO: we might also have let-bindings for the cycle time itself, or
        // arithmetic expressions; this will be a second loop.
        let cloned_first_arg = arenas
            .map(first_arg, Clone::clone)?
            .0
            .item;
        let Expr::Unit(ExprUnit::Number(time)) = cloned_first_arg else {
            return Err(PatternInterpreterError::InvalidTimedStepTime);
        };
        (resolve_second { time, second_arg }, None)
    },
    fn resolve_second(time: CycleTime, second_arg: Index<ExprNode>) {
        if self.get_elem {
            // Construct the timed step.
            value_stack.push(TimedStep(time, second_arg))?;
        } else {
            value_stack.push(time)?;
        }
        (finish, None)
    }
);

impl_frame!(
    /// Given a list of timed steps, uses another frame to find the total length
    /// and then plays the repetitions.
    #[derive(derive_more::Debug, Clone)]
    pub struct PlayTimeCatOrArrangeFrame {
        state: PlayTimeCatOrArrangeState,
        query_mode: QueryMode,
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
        call_args: CallArgs,
        is_time_cat: bool,
    ) {
        (
            pending_total_cycles {
                play_args,
                call_args: call_args.clone(),
                is_time_cat,
            },
            Some(
                ConcatFrame::GetTotalCycles(GetTotalCyclesFrame {
                    state: GetTotalCyclesState::start { call_args },
                })
                .into(),
            ),
        )
    },
    /// Pending the total number of cycles to be calculated from another frame.
    fn pending_total_cycles(
        play_args: PlayElemArgs<()>,
        call_args: CallArgs,
        is_time_cat: bool,
    ) {
        let total_cycle_length = value_stack.pop()?;
        let multiple_length =
            CycleTime::checked_from_int(i32::from(call_args.len()))?;
        (
            in_play_multiple {
                play_multiple: PlayMultiple::new(
                    play_args.clone(),
                    self.query_mode.clone(),
                    TimeCatOrArrangeCycle {
                        multiple_length,
                        total_cycle_length,
                        is_time_cat,
                        iter: call_args.iter(),
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
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
        is_fast: bool,
    ) -> PatternInterpreterResult<Self> {
        if call_args.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        let length = CycleTime::checked_from_int(i32::from(call_args.len()))?;

        Ok(Self(PlayMultiple::new(
            play_args,
            query_mode,
            MultipleOfOneCycle {
                iter: call_args.iter(),
                total_cycle_length: length,
            },
            is_fast,
        )))
    }
}

impl ConcatFrame {
    pub fn new_cat(
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(
            call_args, play_args, query_mode, false,
        )?))
    }

    pub fn new_seq(
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::CatOrSeq(CatOrSeqFrame::new(
            call_args, play_args, query_mode, true,
        )?))
    }

    pub fn new_stack(
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        if call_args.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        Ok(Self::Stack(StackFrame {
            call_args_iter: call_args.iter(),
            play_args,
            query_mode,
        }))
    }

    fn new_time_cat_or_arrange(
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
        is_time_cat: bool,
    ) -> PatternInterpreterResult<Self> {
        if call_args.is_empty() {
            return Err(PatternInterpreterError::MultipleEmpty);
        }

        Ok(Self::TimeCatOrArrange(PlayTimeCatOrArrangeFrame {
            state: PlayTimeCatOrArrangeState::start {
                call_args,
                play_args,
                is_time_cat,
            },
            query_mode,
        }))
    }

    pub fn new_time_cat(
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Self::new_time_cat_or_arrange(call_args, play_args, query_mode, true)
    }

    pub fn new_arrange(
        call_args: CallArgs,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Self::new_time_cat_or_arrange(call_args, play_args, query_mode, false)
    }
}

impl EvaluateFrame for StackFrame {
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        arenas: &impl ExprArenas,
    ) -> InterpreterResult {
        let Some(call_arg) = self
            .call_args_iter
            .next_in_arena(arenas)?
        else {
            return Ok(None);
        };

        Ok(Some(
            QueryFrame::new_non_head(
                call_arg,
                self.play_args.clone(),
                self.query_mode.clone(),
            )?
            .into(),
        ))
    }
}

impl EvaluateFrame for ConcatFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl ExprArenas,
    ) -> InterpreterResult {
        // TODO: Delegate this step.
        match self {
            Self::Stack(frame) => frame.step(value_stack, arenas),
            Self::CatOrSeq(frame) => frame.0.step(value_stack, arenas),
            Self::GetTimedStep(frame) => frame.step(value_stack, arenas),
            Self::TimeCatOrArrange(frame) => frame.step(value_stack, arenas),
            Self::GetTotalCycles(frame) => frame.step(value_stack, arenas),
        }
    }
}
