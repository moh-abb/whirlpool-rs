use core::fmt::Debug;

use combine::Parser;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::many;
use combine::parser;
use combine::sep_end_by1;
use combine::token;
use combine::tokens_cmp;

use crate::ast::note::NoteUnit;
use crate::ast::note::parser::ParseNoteUnitError;
use crate::ast::parser::AstInput;
use crate::ast::parser::AstParser;
use crate::ast::parser::collect::CollectIntoPattern;
use crate::ast::parser::collect::PatternToCollect;
use crate::ast::parser::collect::TimedStepToCollect;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternNode;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::arenas::PatternArenasExt;
use crate::ast::pattern::arenas::PatternPushBackError;
use crate::ast::pattern::arenas::SumCycleLengthError;
use crate::ast::pattern::arenas::timed_step_total_cycle_length;
use crate::ast::pattern::drop::PatternDropAdapter;
use crate::ast::time::CycleTime;
use crate::ast::time::parser::ParseCycleTimeError;
use crate::mem::Arena;
use crate::mem::ArenaError;
use crate::mem::Multiple;
use crate::mem::SharedArenaRef;

#[derive(derive_more::From, Debug, Clone)]
pub enum ParsePatternError {
    ExpectedFullAdapter,
    ExpectedTimedStep,
    ExpectedSetParent,
    ArenaErr(ArenaError),
    CycleTimeErr(ParseCycleTimeError),
    NoteUnitErr(ParseNoteUnitError),
    SumCycleLengthErr(SumCycleLengthError),
    PatternPushBackErr(PatternPushBackError),
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

type ParseResult<T> = Result<T, ParsePatternError>;
type ParsedPattern<'src, A> = PatternDropAdapter<SharedArenaRef<'src, A>>;

pub type ParsePatternResult<'src, A> = ParseResult<ParsedPattern<'src, A>>;

fn leaf<'src, A: PatternArenas, I: AstInput>(
    shared_arena_ref: SharedArenaRef<'src, A>,
) -> impl AstParser<I, ParsePatternResult<'src, A>> {
    let leaf_note = NoteUnit::parser().map(|x| {
        x.map(Pattern::Note)
            .map_err(ParsePatternError::NoteUnitErr)
    });
    let leaf_silence =
        choice([b'~', b'-'].map(token)).map(|_| Ok(Pattern::Silence));

    let push_leaf = move |opt_leaf: ParseResult<Pattern>| {
        opt_leaf.and_then(|leaf| {
            shared_arena_ref
                .clone()
                .push_dropping(leaf)
                .map_err(Into::into)
        })
    };

    leaf_note
        .or(leaf_silence)
        .map(push_leaf)
}

fn whitespace<'src, I: AstInput>() -> impl AstParser<I, ()> {
    many::<(), _, _>(token(b' ').map(|_| ()))
}

fn separator<'src, I: AstInput>() -> impl AstParser<I, ()> {
    between(whitespace(), whitespace(), token(b',')).map(|_| ())
}

fn update_total_cycle_length<'src, A: PatternArenas>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    func_result: ParsePatternResult<'src, A>,
) -> ParsePatternResult<'src, A> {
    // Set the total length of the result.
    let parent_adapter: PatternDropAdapter<_> = func_result?;
    let parent_index = parent_adapter
        .clone_index()
        .ok_or(ParsePatternError::ExpectedFullAdapter)?;

    let mut cloned_arena_ref = shared_arena_ref.clone();
    let parent_multiple =
        cloned_arena_ref.map_mut(parent_index.clone(), |node| {
            let (_, parent_mut_multiple) = timed_step_func_fields(node)?;
            Result::<_, ParsePatternError>::Ok(parent_mut_multiple.clone())
        })??;

    let total_length =
        timed_step_total_cycle_length(parent_multiple, &cloned_arena_ref)?;

    cloned_arena_ref.map_mut(parent_index, |node| {
        let (mut_total_length, _) = timed_step_func_fields(node)?;
        *mut_total_length = total_length;
        Result::<_, ParsePatternError>::Ok(())
    })??;

    Ok(parent_adapter)
}

fn func_str_to_parser<I: AstInput>(
    func_tokens: &'static [u8],
) -> impl AstParser<I, ()> {
    attempt(tokens_cmp(func_tokens, |cx: &u8, cy: u8| *cx == cy).map(|_| ()))
}

fn normal_func<'src, A, I, F, P>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    mut subpattern: F,
) -> impl AstParser<I, ParsePatternResult<'src, A>>
where
    A: PatternArenas + Debug,
    I: AstInput,
    P: AstParser<I, ParsePatternResult<'src, A>>,
    F: FnMut() -> P,
{
    let normal_func_map: [(&[u8], fn(_) -> _); _] = [
        (b"cat", Pattern::Cat),
        (b"seq", Pattern::Seq),
        (b"stack", Pattern::Stack),
    ];

    let normal_func_start = choice(normal_func_map.map(|(func_str, func)| {
        func_str_to_parser(func_str).map(move |_| func)
    }));

    let cloned_arena_ref = shared_arena_ref.clone();
    let normal_func_start_pushing = normal_func_start.map(move |normal_func| {
        cloned_arena_ref
            .clone()
            .push_dropping(normal_func(Multiple::new()))
            .map_err(Into::into)
    });

    normal_func_start_pushing.then(move |opt_parent| {
        let opt_parent_index = CollectIntoPattern::parent_index_of(&opt_parent);
        let parent_and_arena =
            (opt_parent_index.clone(), shared_arena_ref.clone());
        let subpattern_with_parent =
            subpattern().map(move |opt_child| PatternToCollect {
                parent_and_arena: parent_and_arena.clone(),
                opt_child,
            });

        let mut opt_opt_parent = Some(opt_parent);
        between(
            token(b'('),
            token(b')'),
            sep_end_by1::<CollectIntoPattern<_>, _, _, _>(
                subpattern_with_parent,
                separator(),
            )
            .map(move |mut collected| {
                collected.into_result(opt_opt_parent.take())
            }),
        )
    })
}

fn timed_step_func<'src, A, I, F, P>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    mut subpattern: F,
) -> impl AstParser<I, ParsePatternResult<'src, A>>
where
    A: PatternArenas + Debug,
    I: AstInput,
    P: AstParser<I, ParsePatternResult<'src, A>>,
    F: FnMut() -> P,
{
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

    let timed_step_func_start =
        choice(timed_step_func_map.map(|(func_str, func)| {
            func_str_to_parser(func_str).map(move |_| func)
        }));

    let cloned_arena_ref = shared_arena_ref.clone();
    let timed_step_func_start_pushing =
        timed_step_func_start.map(move |timed_step_func| {
            cloned_arena_ref
                .clone()
                .push_dropping(timed_step_func(
                    CycleTime::ZERO,
                    Multiple::new(),
                ))
                .map_err(Into::into)
        });

    timed_step_func_start_pushing.then(move |opt_parent| {
        let timed_step = between(
            token(b'['),
            token(b']'),
            (
                CycleTime::parser(),
                token(b',').map(|_| ()),
                whitespace(),
                subpattern(),
            ),
        );

        let opt_parent_index = CollectIntoPattern::parent_index_of(&opt_parent);
        let parent_and_arena =
            (opt_parent_index.clone(), shared_arena_ref.clone());
        let timed_step_with_parent = timed_step.map(move |opt_timed_step| {
            let (opt_time, (), (), opt_child) = opt_timed_step;
            TimedStepToCollect {
                parent_and_arena: parent_and_arena.clone(),
                opt_time: opt_time.map_err(ParsePatternError::from),
                opt_child,
            }
        });

        let mut opt_opt_parent = Some(opt_parent);
        let cloned_arena_ref = shared_arena_ref.clone();
        between(
            token(b'('),
            token(b')'),
            sep_end_by1::<CollectIntoPattern<_>, _, _, _>(
                timed_step_with_parent,
                separator(),
            )
            .map(move |mut collected| {
                collected.into_result(opt_opt_parent.take())
            })
            .map(move |result| {
                update_total_cycle_length(cloned_arena_ref.clone(), result)
            }),
        )
    })
}

fn choice_pattern<'src, A: PatternArenas + Debug, I: AstInput, RP, F>(
    shared_arena_ref: SharedArenaRef<'src, A>,
    subpattern: F,
) -> impl AstParser<I, ParsePatternResult<'src, A>>
where
    RP: AstParser<I, ParsePatternResult<'src, A>>,
    F: FnMut() -> RP + Clone,
{
    choice((
        normal_func(shared_arena_ref.clone(), subpattern.clone()),
        timed_step_func(shared_arena_ref.clone(), subpattern),
        leaf(shared_arena_ref),
    ))
}

parser! {
    fn recursive_pattern['src, A, I](shared_arena_ref: SharedArenaRef<'src, A>)(I) -> ParsePatternResult<'src, A>
    where [
        I: AstInput,
        A: PatternArenas + Debug,
    ]
    {
        choice_pattern(shared_arena_ref.clone(), || recursive_pattern(shared_arena_ref.clone()))
    }
}

impl<'src, A: PatternArenas + Debug> ParsedPattern<'src, A> {
    pub fn parser<I: AstInput>(
        shared_arena_ref: SharedArenaRef<'src, A>,
    ) -> impl AstParser<I, Result<Self, ParsePatternError>> {
        recursive_pattern(shared_arena_ref)
    }
}
