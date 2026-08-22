use chumsky::Parser;
use chumsky::primitive::choice;
use chumsky::primitive::just;
use chumsky::recursive::recursive;

use crate::ast::NoteUnit;
use crate::ast::Pattern;
use crate::ast::PatternNode;
use crate::ast::parser::AstParser;
use crate::ast::parser::Input;
use crate::ast::parser::Parseable;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::pattern::note::parser::ParseNoteUnitError;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Cow;
use crate::mem::Multiple;
use crate::mem::arena::arena_impl::shared_arena::SharedArenaRef;

#[derive(derive_more::From, Debug)]
pub enum ParsePatternError {
    ArenaErr(ArenaError),
    NoteUnitErr(ParseNoteUnitError),
    ExpectedFullAdapter,
}

impl<'src, 'a, A: PatternArenas>
    Parseable<'src, SharedArenaRef<'src, 'a, PatternNode, A>>
    for PatternDropAdapter<'src, 'a, A>
{
    type Error = ParsePatternError;

    fn parser<I: Input<'src>>(
        shared_arena_ref: SharedArenaRef<'src, 'a, PatternNode, A>,
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
        let _timed_step_func_map: [(&[u8], fn(_, _) -> _); _] =
            [(b"timecat", time_cat_func), (b"arrange", arrange_func)];

        let push_to_arena =
            move |opt_node: Result<Pattern, ParsePatternError>| {
                let node = opt_node?;
                let mut cloned_arena_ref = shared_arena_ref.clone();
                let node_index =
                    cloned_arena_ref.push(PatternNode::new(node))?;
                Result::<_, ParsePatternError>::Ok(PatternDropAdapter(
                    Some(node_index),
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
        let separator = just(b',').padded_by(whitespace);
        let open_arguments = just(b'(');
        let close_arguments = just(b')');

        let normal_func_start = choice(
            normal_func_map.map(|(func_str, func)| just(func_str).to(func)),
        );

        recursive(move |subpattern| {
            let normal_func = normal_func_start
                .map(move |normal_func| {
                    push_to_arena(Ok(normal_func(Multiple::new())))
                })
                .then_ignore(open_arguments)
                .foldl(subpattern.separated_by(separator), move |acc, opt_x| {
                    let parent_adapter = acc?;
                    let parent_index = parent_adapter
                        .0
                        .clone()
                        .ok_or(ParsePatternError::ExpectedFullAdapter)?;

                    let mut child_adapter: PatternDropAdapter<_> = opt_x?;
                    let child_index = child_adapter
                        .0
                        .take()
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
                })
                .then_ignore(close_arguments);

            normal_func.or(leaf)
        })
    }
}
