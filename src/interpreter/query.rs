use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::interpreter::concat::ConcatFrame;
use crate::interpreter::error::PatternInterpreterError;
use crate::interpreter::frame::EvaluateFrame;
use crate::interpreter::frame::InterpreterFrame;
use crate::interpreter::frame::InterpreterResult;
use crate::interpreter::leaf::LeafFrame;
use crate::interpreter::props::PlayElemArgs;
use crate::interpreter::value::ValueStack;
use crate::mem::Index;

#[derive(derive_new::new, Debug, Clone)]
pub(super) struct QueryFrame {
    play_args: PlayElemArgs<Index<PatternNode>>,
    #[new(value = "false")]
    finished: bool,
}

impl EvaluateFrame for QueryFrame {
    fn step(
        &mut self,
        _: &mut impl ValueStack,
        arenas: &impl PatternArenas,
    ) -> InterpreterResult {
        // Check if we are done.
        if self.finished {
            return Ok(None);
        }
        let index = self.play_args.elem.clone();
        let play_args = self.play_args.clone().map(|_| ());
        let cloned_pattern = arenas.map(index, Clone::clone)?.pattern;
        let next_frame: InterpreterFrame = match &cloned_pattern {
            Pattern::Cat(multiple) => {
                ConcatFrame::new_cat(multiple, play_args)?.into()
            }
            Pattern::Seq(multiple) => {
                ConcatFrame::new_seq(multiple, play_args)?.into()
            }
            Pattern::Stack(multiple) => {
                ConcatFrame::new_stack(multiple, play_args)?.into()
            }
            Pattern::TimeCat { multiple, .. } => {
                ConcatFrame::new_time_cat(multiple, play_args)?.into()
            }
            Pattern::Arrange { multiple, .. } => {
                ConcatFrame::new_arrange(multiple, play_args)?.into()
            }
            Pattern::TimedStep(_) => {
                // We shouldn't be querying a timed step without using the
                // parent TimeCat or Arrange.
                return Err(PatternInterpreterError::ExpectedNormalPattern);
            }
            Pattern::Note(note_unit) => {
                LeafFrame::new_note(note_unit, play_args)?.into()
            }
            Pattern::Silence => LeafFrame::new_silence()?.into(),
        };
        // Indicate that we will delete this frame after the child frame exits.
        self.finished = true;
        Ok(Some(next_frame))
    }
}
