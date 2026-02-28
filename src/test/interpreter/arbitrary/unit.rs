use proptest::prelude::Strategy;
use proptest::prelude::any;

use crate::arena::Arena;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::test::interpreter::arbitrary::PlayNoteSequence;
use crate::test::interpreter::arbitrary::repeated::arb_chunk_count;
use crate::test::interpreter::arbitrary::repeated::arb_end_offset_and_multiplier;
use crate::test::interpreter::arbitrary::repeated::chunked_repeated_unit;
use crate::test::interpreter::arbitrary::repeated::repeated_unit;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::StrategyWithArena;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;

struct UnitSequenceStrategy;
impl StrategyWithArena<NoteSequence> for UnitSequenceStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        (any::<NoteUnit>(), arb_end_offset_and_multiplier()).prop_map(
            |(note_unit, (end_time, offset, multiplier))| {
                ArenasTo::new(move |arenas: &Arenas| {
                    let head = arenas
                        .get_pattern_arena()
                        .alloc(Pattern::Note(note_unit))?;
                    repeated_unit(
                        head,
                        |_| note_unit,
                        end_time,
                        offset,
                        multiplier,
                    )
                })
            },
        )
    }
}

struct ChunkedUnitSequenceStrategy;
impl StrategyWithArena<NoteSequence> for ChunkedUnitSequenceStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        (arb_chunk_count(), any::<NoteUnit>(), arb_end_offset_and_multiplier())
            .prop_map(
                |(chunk_count, note_unit, (end_time, offset, multiplier))| {
                    ArenasTo::new(move |arenas: &Arenas| {
                        let head = arenas
                            .get_pattern_arena()
                            .alloc(Pattern::Note(note_unit))?;
                        chunked_repeated_unit(
                            head,
                            |_| note_unit,
                            chunk_count,
                            end_time,
                            offset,
                            multiplier,
                        )
                    })
                },
            )
    }
}

#[test]
fn can_play_complete_time_interval() {
    with_regenerated_arenas::<
        _,
        PlayNoteSequence,
        GrowableArenas,
        UnitSequenceStrategy,
    >()
}

#[test]
fn can_play_chunked_time_interval() {
    with_regenerated_arenas::<
        _,
        PlayNoteSequence,
        GrowableArenas,
        ChunkedUnitSequenceStrategy,
    >()
}
