use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::StringLiteral;
use crate::ast::expr::builtin::BuiltinCall;
use crate::ast::expr::call_arg::CallArg;
use crate::ast::expr::call_arg::CallArgs;
use crate::ast::expr::ident::Identifier;
use crate::ast::expr::linked::ExprArenas;
use crate::ast::note::Note;
use crate::ast::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::impl_frame;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::error::PatternInterpreterResult;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::leaf::LeafFrame;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::value::ValueStack;
use crate::mem::Index;

/// Determines how elements such as identifiers should be treated while
/// querying: since we might have "C" be both a chord and a note, handle this
/// ambiguity by passing it as an argument while querying (so that `.note()`
/// will set this mode and resolve the ambiguity).
#[derive(Debug, Clone)]
pub enum QueryMode {
    Note,
}

#[derive(Debug, Clone)]
pub enum QueryFrame {
    QueryExpr(QueryExprFrame),
}

impl_frame!(
    /// Queries the expression at the given head index.
    /// If `query_leaf` is true, then this frame will attempt to treat
    /// identifiers as both references to a previous let-binding, or note
    /// leaves.
    /// Otherwise, when recursing via let-bindings, if we encounter a number
    /// or an identifier that has no binding, we treat this as an error.
    #[derive(derive_new::new, Debug, Clone)]
    #[new(visibility = "")]
    pub struct QueryExprFrame {
        state: QueryExprState,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
        query_leaf: bool,
    },
    [] = [] + [],
    /// State for querying frame.
    #[derive(Debug, Clone)]
    enum QueryExprState {},
    [] = [],
    [self, _value_stack, arenas],
    /// The start state, before flattening the cases of [CallArg].
    fn start(call_arg: CallArg) {
        match call_arg {
            CallArg::Ident(ident) => (in_ident { ident }, None),
            CallArg::Index(index) => (in_index { index }, None),
        }
    },
    /// Resolves the current identifier to find the expression's index
    /// (or a note).
    fn in_ident(ident: Identifier) {
        // TODO: handle let-bindings (and local scoping in a lambda).
        // For now, just treat this as a leaf.

        let bytes = ident.as_ref().as_slice()?;

        // Check for silence.
        if matches!(bytes, b"~" | b"-") {
            (finish, Some(LeafFrame::new_silence()?.into()))
        } else {
            let note_unit = match self.query_mode {
                QueryMode::Note => NoteUnit::Note(Note::resolve(bytes)?),
            };

            let frame = LeafFrame::new_unit(
                note_unit,
                self.play_args.clone(),
                self.query_mode.clone(),
            )?
            .into();

            (finish, Some(frame))
        }
    },
    /// Dereferences the index of the expression to handle the cases.
    fn in_index(index: Index<ExprNode>) {
        let cloned_expr = arenas.map(index, Clone::clone)?.0.item;
        match cloned_expr {
            Expr::Call { is_receiver: _, name, args } => {
                let call_args = CallArgs::new(args);
                (in_call { name, call_args }, None)
            }
            Expr::Lambda { .. } => {
                return Err(PatternInterpreterError::UnexpectedLambdaInQuery);
            }
            Expr::Array { .. } => {
                return Err(PatternInterpreterError::UnexpectedArrayInQuery);
            }
            Expr::Unit(expr_unit) => match expr_unit {
                ExprUnit::Ident(ident) => (in_ident { ident }, None),
                ExprUnit::Number(number) => (in_number { number }, None),
                ExprUnit::String(string) => {
                    (in_string_literal { string }, None)
                }
            },
        }
    },
    /// Inside a call, so resolve it.
    fn in_call(name: Identifier, call_args: CallArgs) {
        // TODO: handle `register` bindings.
        // For now, just treat this as a builtin function.
        let name_bytes = name.as_ref().as_slice()?;
        let make_builtin = match BuiltinCall::resolve(name_bytes)? {
            BuiltinCall::Cat => ConcatFrame::new_cat,
            BuiltinCall::Seq => ConcatFrame::new_seq,
            BuiltinCall::Stack => ConcatFrame::new_stack,
            BuiltinCall::TimeCat => ConcatFrame::new_time_cat,
            BuiltinCall::Arrange => ConcatFrame::new_arrange,
        };

        let frame = make_builtin(
            call_args,
            self.play_args.clone(),
            self.query_mode.clone(),
        )?
        .into();

        (finish, Some(frame))
    },
    fn in_number(number: CycleTime) {
        if !self.query_leaf {
            return Err(PatternInterpreterError::UnexpectedNumberInQuery);
        }

        let frame = LeafFrame::new_unit(
            NoteUnit::Number(number),
            self.play_args.clone(),
            self.query_mode.clone(),
        )?
        .into();

        (finish, Some(frame))
    },
    fn in_string_literal(string: StringLiteral) {
        if !self.query_leaf {
            return Err(PatternInterpreterError::UnexpectedNumberInQuery);
        }

        let frame = LeafFrame::new_unit(
            NoteUnit::String(string),
            self.play_args.clone(),
            self.query_mode.clone(),
        )?
        .into();

        (finish, Some(frame))
    }
);

impl QueryFrame {
    pub fn new_head(
        call_arg: CallArg,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::QueryExpr(QueryExprFrame::new(
            QueryExprState::start { call_arg },
            play_args,
            query_mode,
            false,
        )))
    }

    pub fn new_non_head(
        call_arg: CallArg,
        play_args: PlayElemArgs<()>,
        query_mode: QueryMode,
    ) -> PatternInterpreterResult<Self> {
        Ok(Self::QueryExpr(QueryExprFrame::new(
            QueryExprState::start { call_arg },
            play_args,
            query_mode,
            true,
        )))
    }
}

impl EvaluateFrame for QueryFrame {
    fn step(
        &mut self,
        value_stack: &mut impl ValueStack,
        arenas: &impl ExprArenas,
    ) -> InterpreterResult {
        // TODO: Automatically delegate this step.
        match self {
            Self::QueryExpr(frame) => frame.step(value_stack, arenas),
        }
    }
}
