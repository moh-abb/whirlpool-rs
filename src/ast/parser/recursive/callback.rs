use core::mem::replace;

use combine::ParseError;
use combine::ParseResult;
use combine::Parser;
use combine::StdParseResult;
use combine::StreamOnce;
use combine::error::Commit;
use combine::error::Tracked;
use combine::parser::function::EnvParser;
use combine::parser::function::env_parser;
use combine::stream::ResetStream;
use derive_where::derive_where;

use crate::ast::parser::AstInput;
use crate::ast::parser::recursive::error::RecError;
use crate::ast::parser::recursive::error::RecResult;
use crate::ast::parser::recursive::state::RecOutput;
use crate::mem::Arena;
use crate::mem::ArenaItem;
use crate::mem::Index;
use crate::mem::SharedArenaRef;

/// Used to manage handling one frame of a recursive parse attempt.
/// The initial attempt to partially parse this expression will update the
/// shared recursive state from [RecOutput::Start] to [RecOutput::Pending],
/// then immediately fail.
/// Then the recursive combinator will push a new frame and update the shared
/// recursive state from [RecOutput::Pending] to [RecOutput::Recursing].
/// When the top recursive frame completes (i.e., returns
/// `ParseResult::CommitOk(Result::<_, RecError>::Ok(T))`), the top frame's
/// shared state should be [RecOutput::Start] and the frame below it will be
/// updated from [RecOutput::Recursing] to [RecOutput::Complete] with the value
/// from `T`.
#[derive_where(Clone)]
pub struct RecCallback<'o, I, T, TA>(RecCbInner<'o, I, T, TA>)
where
    I: AstInput,
    T: ArenaItem,
    TA: Arena<RecOutput<I, T>>;

#[derive_where(Clone)]
pub struct RecCallbackData<'o, I: AstInput, T, TA>(
    Index<RecOutput<I, T>>,
    SharedArenaRef<'o, TA>,
);

type RecCbInner<'o, I, T, TA> =
    EnvParser<RecCallbackData<'o, I, T, TA>, I, RecResult<T>>;

impl<'o, I, T, TA> RecCallback<'o, I, T, TA>
where
    I: AstInput,
    T: ArenaItem,
    TA: Arena<RecOutput<I, T>>,
{
    fn get_parser(
        callback_data: RecCallbackData<'o, I, T, TA>,
        input: &mut I,
    ) -> StdParseResult<RecResult<T>, I> {
        let RecCallbackData(index, arena) = callback_data;

        let make_error = |message: &'static str| {
            let mut error = <I as StreamOnce>::Error::empty(input.position());
            error.add_message(message);
            StdParseResult::<RecResult<T>, I>::Err(Commit::Peek(Tracked::from(
                error,
            )))
        };

        arena
            .clone()
            .map_mut(index.clone(), |output| {
                let taken_output = replace(output, RecOutput::Start);
                let (next_output, parser) = match taken_output {
                    RecOutput::Start => (
                        RecOutput::Pending(input.checkpoint()),
                        make_error("First try of recursive callback"),
                    ),
                    RecOutput::Pending(c) => (
                        RecOutput::Pending(c),
                        make_error("Second try of recursive callback"),
                    ),
                    RecOutput::Recursing => (
                        RecOutput::Recursing,
                        make_error("Halfway through a recursing frame"),
                    ),
                    RecOutput::Complete(item) => {
                        (RecOutput::Start, Ok((Ok(item), Commit::Peek(()))))
                    }
                };
                *output = next_output;
                parser
            })
            .unwrap_or_else(|error| {
                Ok((Err(RecError::from(error)), Commit::Commit(())))
            })
    }

    /// Constructs a callback with an index to the output, assuming this
    /// already contains a valid entry initially set to [RecOutput::Start].
    pub(super) fn new(
        output_index: Index<RecOutput<I, T>>,
        output_arena: SharedArenaRef<'o, TA>,
    ) -> Self {
        Self(env_parser(
            RecCallbackData(output_index, output_arena),
            Self::get_parser,
        ))
    }
}

impl<'o, I: AstInput, T, TA> Parser<I> for RecCallback<'o, I, T, TA>
where
    I: AstInput,
    T: ArenaItem,
    TA: Arena<RecOutput<I, T>>,
{
    type Output = <RecCbInner<'o, I, T, TA> as Parser<I>>::Output;
    type PartialState = <RecCbInner<'o, I, T, TA> as Parser<I>>::PartialState;

    fn parse_lazy(
        &mut self,
        input: &mut I,
    ) -> ParseResult<Self::Output, <I as StreamOnce>::Error> {
        self.0.parse_lazy(input)
    }
}
