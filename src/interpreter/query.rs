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
use crate::mem::Index;
use crate::synth::scheduler::UnitScheduler;

#[derive(Debug, derive_new::new)]
pub(super) struct QueryFrame {
    play_args: PlayElemArgs<Index<PatternNode>>,
    #[new(value = "false")]
    finished: bool,
}

impl<'a, Arenas: PatternArenas> EvaluateFrame<'a, Arenas> for QueryFrame {
    fn step(
        &mut self,
        _: &mut impl UnitScheduler,
        arenas: &'a Arenas,
    ) -> InterpreterResult<'a, Arenas> {
        // Check if we are done.
        if self.finished {
            return Ok(None);
        }
        let index = self.play_args.elem.clone();
        let play_args = self.play_args.clone().map(|_| ());
        let cloned_pattern = arenas.map(index, Clone::clone)?.pattern;
        let next_frame: InterpreterFrame<'a, Arenas> = match &cloned_pattern {
            Pattern::Cat(multiple) => {
                ConcatFrame::new_cat(multiple, arenas, play_args)?.into()
            }
            Pattern::Seq(multiple) => {
                ConcatFrame::new_seq(multiple, arenas, play_args)?.into()
            }
            Pattern::Stack(multiple) => {
                ConcatFrame::new_stack(multiple, arenas, play_args)?.into()
            }
            Pattern::TimeCat { total_cycle_length, multiple } => {
                ConcatFrame::new_time_cat(
                    multiple,
                    total_cycle_length,
                    arenas,
                    play_args,
                )?
                .into()
            }
            Pattern::Arrange { total_cycle_length, multiple } => {
                ConcatFrame::new_arrange(
                    multiple,
                    total_cycle_length,
                    arenas,
                    play_args,
                )?
                .into()
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
