use chumsky::Parser;
use chumsky::primitive::choice;
use chumsky::primitive::just;
use chumsky::recursive::recursive;

use crate::ast::CycleTime;
use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::TimedStep;
use crate::ast::parser::AstParser;
use crate::ast::parser::Input;
use crate::ast::parser::Parseable;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::SumCycleLengthError;
use crate::ast::pattern::arenas::timed_step_total_cycle_length;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::note::parser::ParseNoteUnitError;
use crate::ast::time::parser::ParseCycleTimeError;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Cow;
use crate::mem::Multiple;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;

#[derive(derive_more::From, Debug)]
pub enum ParsePatternError {
    ExpectedFullAdapter,
    ExpectedTimedStep,
    ArenaErr(ArenaError),
    CycleTimeErr(ParseCycleTimeError),
    NoteUnitErr(ParseNoteUnitError),
    SumCycleLengthErr(SumCycleLengthError),
}

fn timed_step_func_fields<'a>(
    node: &'a mut PatternNode,
) -> Result<(&'a mut CycleTime, &'a mut Multiple<PatternNode>), ParsePatternError>
{
    match &mut node.pattern {
        Pattern::TimeCat { total_cycle_length, multiple }
        | Pattern::Arrange { total_cycle_length, multiple } => {
            Ok((total_cycle_length, multiple))
        }
        _ => Err(ParsePatternError::ExpectedTimedStep),
    }
}

impl<'src, A: PatternArenas>
    Parseable<'src, SharedArenaRef<'src, PatternNode, A>>
    for PatternDropAdapter<SharedArenaRef<'src, PatternNode, A>>
{
    type Error = ParsePatternError;

    fn parser<I: Input<'src>>(
        shared_arena_ref: SharedArenaRef<'src, PatternNode, A>,
    ) -> impl AstParser<'src, I, Result<Self, Self::Error>> {
        let normal_func_map: [(&[u8], fn(_) -> _); _] = [
            (b"cat", Pattern::Cat),
            (b"seq", Pattern::Seq),
            (b"stack", Pattern::Stack),
        ];

        let time_cat_func = |total_cycle_length, multiple| Pattern::TimeCat {
            total_cycle_length,
            multiple,
        };
        let arrange_func = |total_cycle_length, multiple| Pattern::Arrange {
            total_cycle_length,
            multiple,
        };
        let timed_step_func_map: [(&[u8], fn(_, _) -> _); _] = [
            (b"timecat", time_cat_func),
            (b"timeCat", time_cat_func),
            (b"stepcat", time_cat_func),
            (b"arrange", arrange_func),
        ];

        let push_to_arena =
            move |opt_node: Result<Pattern, ParsePatternError>| {
                let node = opt_node?;
                let mut cloned_arena_ref = shared_arena_ref.clone();
                let node_index =
                    cloned_arena_ref.push(PatternNode::new(node))?;
                Result::<_, ParsePatternError>::Ok(PatternDropAdapter::new(
                    node_index,
                    cloned_arena_ref,
                ))
            };

        let leaf_note = NoteUnit::parser(()).map(|x| {
            x.map(Pattern::Note)
                .map_err(ParsePatternError::NoteUnitErr)
        });
        let leaf_silence =
            choice([b'~', b'-'].map(just)).map(|_| Ok(Pattern::Silence));
        let leaf = leaf_note
            .or(leaf_silence)
            .map(push_to_arena.clone());

        let whitespace = just(b' ').repeated();
        let separator = just(b',').padded_by(whitespace.clone());
        let open_arguments = just(b'(');
        let close_arguments = just(b')');

        let normal_func_start = choice(
            normal_func_map.map(|(func_str, func)| just(func_str).to(func)),
        );
        let timed_step_func_start = choice(
            timed_step_func_map.map(|(func_str, func)| just(func_str).to(func)),
        );

        let cons_pattern = move |acc: _, opt_x: _| {
            let parent_adapter: PatternDropAdapter<_> = acc?;
            let parent_index = parent_adapter
                .clone_index()
                .ok_or(ParsePatternError::ExpectedFullAdapter)?;

            let mut child_adapter: PatternDropAdapter<_> = opt_x?;
            let child_index = child_adapter
                .take_index()
                .ok_or(ParsePatternError::ExpectedFullAdapter)?;

            let mut shared_arena_ref_1 = shared_arena_ref.clone();
            let mut shared_arena_ref_2 = shared_arena_ref.clone();
            Multiple::push_back(
                &mut shared_arena_ref_1,
                &mut shared_arena_ref_2,
                parent_index,
                Cow::Indexed(child_index),
            )?;

            Result::<_, ParsePatternError>::Ok(parent_adapter)
        };

        let create_timed_step = move |opt_time, opt_x| {
            let time = opt_time?;
            let mut child_adapter: PatternDropAdapter<_> = opt_x?;
            let child_index = child_adapter
                .clone_index()
                .ok_or(ParsePatternError::ExpectedFullAdapter)?;

            let timed_step_adapter = push_to_arena(Ok(Pattern::TimedStep(
                TimedStep(time, Multiple::new()),
            )))?;

            let timed_step_index = timed_step_adapter
                .clone_index()
                .ok_or(ParsePatternError::ExpectedFullAdapter)?;

            let mut shared_arena_ref_1 = shared_arena_ref.clone();
            let mut shared_arena_ref_2 = shared_arena_ref.clone();
            Multiple::push_back(
                &mut shared_arena_ref_1,
                &mut shared_arena_ref_2,
                timed_step_index,
                Cow::Indexed(child_index),
            )?;

            // Avoid dropping the newly-linked child.
            child_adapter.take_index();

            Result::<_, ParsePatternError>::Ok(timed_step_adapter)
        };

        let update_total_cycle_length = move |func_result| {
            // Set the total length of the result.

            let parent_adapter: PatternDropAdapter<_> = func_result?;
            let parent_index = parent_adapter
                .clone_index()
                .ok_or(ParsePatternError::ExpectedFullAdapter)?;

            let mut cloned_arena_ref = shared_arena_ref.clone();
            let parent_multiple =
                cloned_arena_ref.map_mut(parent_index.clone(), |node| {
                    let (_, parent_mut_multiple) =
                        timed_step_func_fields(node)?;
                    Result::<_, ParsePatternError>::Ok(
                        parent_mut_multiple.clone(),
                    )
                })??;

            let total_length = timed_step_total_cycle_length(
                parent_multiple,
                &cloned_arena_ref,
            )?;

            cloned_arena_ref.map_mut(parent_index.clone(), |node| {
                let (mut_total_length, _) = timed_step_func_fields(node)?;
                *mut_total_length = total_length;
                Result::<_, ParsePatternError>::Ok(())
            })??;

            Result::<_, ParsePatternError>::Ok(parent_adapter)
        };

        recursive(move |subpattern| {
            let normal_func = normal_func_start
                .map(move |normal_func| {
                    push_to_arena(Ok(normal_func(Multiple::new())))
                })
                .then_ignore(open_arguments)
                .foldl(
                    subpattern
                        .clone()
                        .separated_by(separator.clone()),
                    cons_pattern,
                )
                .then_ignore(close_arguments);

            let timed_step = just(b'[')
                .ignore_then(CycleTime::parser(()))
                .then_ignore(separator.clone())
                .then(subpattern)
                .then_ignore(just(b']'))
                .map(move |(opt_time, opt_x)| {
                    create_timed_step(opt_time, opt_x)
                });

            let timed_step_func = timed_step_func_start
                .map(move |timed_step_func| {
                    push_to_arena(Ok(timed_step_func(
                        CycleTime::ZERO,
                        Multiple::new(),
                    )))
                })
                .then_ignore(open_arguments)
                .foldl(
                    timed_step
                        .clone()
                        .separated_by(separator),
                    cons_pattern,
                )
                .then_ignore(close_arguments)
                .map(update_total_cycle_length);

            normal_func.or(timed_step_func).or(leaf)
        })
    }
}
