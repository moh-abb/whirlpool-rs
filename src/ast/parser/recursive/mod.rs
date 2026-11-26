/// Denotes the type for the number of partial parse attempts
/// during a recursive parse.
type RecIterations = usize;

use core::debug_assert_matches;
use core::mem::replace;
use core::ops::ControlFlow;
use core::ops::ControlFlow::Break;
use core::ops::ControlFlow::Continue;

use combine::ParseResult;
use combine::ParseResult::CommitErr;
use combine::ParseResult::CommitOk;
use combine::ParseResult::PeekErr;
use combine::ParseResult::PeekOk;
use combine::Parser;
use combine::StreamOnce;

use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::recursive::callback::RecCallback;
use crate::ast::parser::recursive::error::RecError;
use crate::ast::parser::recursive::error::RecResult;
use crate::ast::parser::recursive::state::RecFrame;
use crate::ast::parser::recursive::state::RecOutput;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::SharedArenaRef;
use crate::mem::Stack;
use crate::mem::debug_unwrap;
use crate::mem::structures::stack::StackChain;

pub mod callback;
pub mod error;
pub mod state;

pub struct ArenaRecursive<'o, 'f, I, T, PA, TA, P, F>
where
    I: AstInput,
    T: ArenaItem + 'o + 'f,
    TA: Arena<RecOutput<I, T>> + 'o,
    PA: Arena<StackChain<RecFrame<I, T, P>>> + 'f,
    P: AstParser<I, RecResult<T>> + 'o,
    F: FnMut(RecCallback<'o, I, T, TA>) -> P,
{
    gen_frame: F,
    output_arena: SharedArenaRef<'o, TA>,
    cur_iterations: RecIterations,
    max_iterations: RecIterations,
    frame_stack: Stack<RecFrame<I, T, P>, SharedArenaRef<'f, PA>>,
}

pub fn arena_recursive<'o, 'f, I, T, PA, TA, P, F>(
    gen_frame: F,
    output_arena: SharedArenaRef<'o, TA>,
    frame_arena: SharedArenaRef<'f, PA>,
    max_iterations: RecIterations,
) -> ArenaRecursive<'o, 'f, I, T, PA, TA, P, F>
where
    I: AstInput,
    T: ArenaItem + 'o + 'f,
    TA: Arena<RecOutput<I, T>> + 'o,
    PA: Arena<StackChain<RecFrame<I, T, P>>> + 'f,
    P: AstParser<I, RecResult<T>> + 'o,
    F: FnMut(RecCallback<'o, I, T, TA>) -> P,
{
    ArenaRecursive {
        gen_frame,
        output_arena,
        cur_iterations: 0,
        max_iterations,
        frame_stack: Stack::new(frame_arena),
    }
}

type ProcessedParseResult<T, I> = RecResult<
    ControlFlow<
        // Break with the parse result.
        ParseResult<RecResult<T>, <I as StreamOnce>::Error>,
        // Continue iteration. At this stage, we don't consider when the parser
        // needs further attempts to gather more values. In that case, we might
        // need to consider using an optional output instead of `()`.
        (),
    >,
>;

impl<'o, 'f, I, T, PA, TA, P, F> ArenaRecursive<'o, 'f, I, T, PA, TA, P, F>
where
    I: AstInput,
    T: ArenaItem + 'o + 'f,
    TA: Arena<RecOutput<I, T>> + 'o,
    PA: Arena<StackChain<RecFrame<I, T, P>>> + 'f,
    P: AstParser<I, RecResult<T>> + 'o,
    F: FnMut(RecCallback<'o, I, T, TA>) -> P,
{
    fn push_frame(&mut self) -> RecResult<()> {
        let output_index = self
            .output_arena
            .push(RecOutput::Start)?;
        let callback =
            RecCallback::new(output_index.clone(), self.output_arena.clone());
        self.frame_stack
            .push(RecFrame {
                parser: (self.gen_frame)(callback),
                state: Default::default(),
                output_index: output_index.clone(),
            })
            .or_else(|error| {
                // Free the allocated output.
                let _ = self.output_arena.take(output_index)?;
                Err(error)
            })?;
        Ok(())
    }

    fn pop_frame(&mut self) -> RecResult<()> {
        let frame = self
            .frame_stack
            .pop()?
            .ok_or(RecError::StackEmpty)?;
        let _ = self
            .output_arena
            .take(frame.output_index)?;
        Ok(())
    }

    fn clear_frames(&mut self) -> RecResult<()> {
        while !self.frame_stack.is_empty() {
            self.pop_frame()?;
        }
        Ok(())
    }

    fn parse_top_frame(
        &mut self,
        input: &mut I,
    ) -> RecResult<ParseResult<RecResult<T>, I::Error>> {
        self.frame_stack
            .map_mut(|RecFrame { parser, state, output_index: _ }| {
                parser.parse_stream_partial(input, state)
            })?
            .ok_or(RecError::StackEmpty)
    }

    fn next_frame_checkpoint(&mut self) -> RecResult<Option<I::Checkpoint>> {
        let output_index = self
            .frame_stack
            .map(|frame| frame.output_index.clone())?
            .ok_or(RecError::StackEmpty)?;
        self.output_arena
            .map_mut(output_index, |output| {
                let taken_output = replace(output, RecOutput::Start);
                let (opt_checkpoint, next_output) = match taken_output {
                    RecOutput::Start => (None, RecOutput::Start),
                    RecOutput::Pending(checkpoint) => {
                        (Some(checkpoint), RecOutput::Recursing)
                    }
                    RecOutput::Recursing => {
                        debug_assert!(false);
                        (None, RecOutput::Recursing)
                    }
                    RecOutput::Complete(item) => {
                        (None, RecOutput::Complete(item))
                    }
                };
                *output = next_output;
                opt_checkpoint
            })
            .map_err(RecError::from)
    }

    fn set_rec_output(&mut self, item: T) -> RecResult<()> {
        let output_index = self
            .frame_stack
            .map(|frame| frame.output_index.clone())?
            .ok_or(RecError::StackEmpty)?;
        self.output_arena
            .map_mut(output_index, |old_output| {
                debug_assert_matches!(old_output, RecOutput::Recursing);
                *old_output = RecOutput::Complete(item)
            })
            .map_err(RecError::from)
    }

    fn process_loop_iter(
        &mut self,
        input: &mut I,
    ) -> ProcessedParseResult<T, I> {
        let ok_value = |this: &mut Self,
                        opt_item: RecResult<T>,
                        f: fn(_) -> ParseResult<_, _>| {
            this.pop_frame()?;
            let result = if this.frame_stack.is_empty() {
                Break(f(opt_item))
            } else {
                let item = opt_item?;
                this.set_rec_output(item)?;
                Continue(())
            };
            ProcessedParseResult::<T, I>::Ok(result)
        };

        let err_value =
            |this: &mut Self, input_mut: &mut I, error: ParseResult<_, _>| {
                // Check the state of the shared recursive output.
                let result =
                    if let Some(checkpoint) = this.next_frame_checkpoint()? {
                        input_mut
                            .reset(checkpoint)
                            .map_err(|_| RecError::ResetFailed)?;
                        this.push_frame()?;
                        Continue(())
                    } else {
                        Break(error)
                    };
                ProcessedParseResult::<T, I>::Ok(result)
            };

        match self.parse_top_frame(input)? {
            CommitOk(opt_item) => ok_value(self, opt_item, CommitOk),
            PeekOk(opt_item) => ok_value(self, opt_item, PeekOk),
            CommitErr(error) => err_value(self, input, CommitErr(error)),
            PeekErr(tracked) => err_value(self, input, PeekErr(tracked)),
        }
    }
}

impl<'o, 'f, I, T, PA, TA, P, F> Drop
    for ArenaRecursive<'o, 'f, I, T, PA, TA, P, F>
where
    I: AstInput,
    T: ArenaItem + 'o + 'f,
    TA: Arena<RecOutput<I, T>> + 'o,
    PA: Arena<StackChain<RecFrame<I, T, P>>> + 'f,
    P: AstParser<I, RecResult<T>> + 'o,
    F: FnMut(RecCallback<'o, I, T, TA>) -> P,
{
    fn drop(&mut self) {
        debug_unwrap(self.clear_frames())
    }
}

impl<'o, 'f, I, T, PA, TA, P, F> Parser<I>
    for ArenaRecursive<'o, 'f, I, T, PA, TA, P, F>
where
    I: AstInput,
    T: ArenaItem + 'o + 'f,
    TA: Arena<RecOutput<I, T>> + 'o,
    PA: Arena<StackChain<RecFrame<I, T, P>>> + 'f,
    P: AstParser<I, RecResult<T>> + 'o,
    F: FnMut(RecCallback<'o, I, T, TA>) -> P,
{
    type Output = Result<T, RecError>;

    type PartialState = ();

    fn parse_lazy(
        &mut self,
        input: &mut I,
    ) -> ParseResult<Self::Output, <I as StreamOnce>::Error> {
        if let Err(error) = self.push_frame() {
            return CommitOk(Err(error));
        }

        let output = loop {
            // Check if the iteration limit has been reached.
            if self.cur_iterations >= self.max_iterations {
                break CommitOk(Err(RecError::MaximumIterations(
                    self.max_iterations,
                )));
            }
            self.cur_iterations = self.cur_iterations.saturating_add(1);

            match self.process_loop_iter(input) {
                Ok(Continue(())) => continue,
                Ok(Break(parse_result)) => break parse_result,
                Err(error) => break CommitOk(Err(error)),
            }
        };

        debug_unwrap(self.clear_frames());
        output
    }
}
