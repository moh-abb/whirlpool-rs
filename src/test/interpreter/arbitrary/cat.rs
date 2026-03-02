use core::num::NonZeroU16;

use proptest::prelude::Strategy;

use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::time::CycleTime;
use crate::test::interpreter::arbitrary::PlayNoteSequence;
use crate::test::interpreter::arbitrary::repeated::arb_chunk_count;
use crate::test::interpreter::arbitrary::repeated::arb_end_offset_and_multiplier;
use crate::test::interpreter::arbitrary::repeated::chunked_repeated_unit;
use crate::test::interpreter::sequence::NoteSequence;
use crate::test::pattern::arbitrary::arenas_to::ArenasTo;
use crate::test::pattern::arbitrary::cat::arb_cat_of_units;
use crate::test::pattern::arena_alloc::GrowableArenas;
use crate::test::pattern::arena_alloc::StrategyWithArena;
use crate::test::pattern::arena_alloc::with_regenerated_arenas;

fn arb_note_sequence_from_cat_of_units<Arenas: PatternArenas + 'static>(
    chunk_count: NonZeroU16,
) -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
    (arb_cat_of_units(), arb_end_offset_and_multiplier()).prop_map(
        move |(arenas_to, (end_time, offset, multiplier))| {
            ArenasTo::new(move |arenas: &Arenas| {
                let (head, units) = arenas_to.call(arenas)?;
                let get_unit = |start_time: CycleTime| {
                    assert_eq!(
                        start_time.frac(),
                        CycleTime::ZERO,
                        "Cat unit start time should be aligned"
                    );
                    let start_time_index = usize::try_from(start_time.to_int())
                        .expect("Cat unit start time should be positive");
                    units[start_time_index % units.len()]
                };

                chunked_repeated_unit(
                    head,
                    get_unit,
                    chunk_count,
                    end_time,
                    offset,
                    multiplier,
                )
            })
        },
    )
}

struct CatOfUnitsStrategy;
impl StrategyWithArena<NoteSequence> for CatOfUnitsStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        arb_note_sequence_from_cat_of_units(NonZeroU16::new(1).unwrap())
    }
}

struct ChunkedCatOfUnitsStrategy;
impl StrategyWithArena<NoteSequence> for ChunkedCatOfUnitsStrategy {
    fn item_strategy<Arenas: PatternArenas + 'static>()
    -> impl Strategy<Value = ArenasTo<Arenas, NoteSequence>> {
        arb_chunk_count().prop_flat_map(arb_note_sequence_from_cat_of_units)
    }
}

#[test]
fn can_play_complete_time_interval() {
    with_regenerated_arenas::<
        _,
        PlayNoteSequence,
        GrowableArenas,
        CatOfUnitsStrategy,
    >()
}

#[test]
fn can_play_chunked_time_interval() {
    with_regenerated_arenas::<
        _,
        PlayNoteSequence,
        GrowableArenas,
        ChunkedCatOfUnitsStrategy,
    >()
}
