use core::cell::RefCell;

use mockall::predicate;

use crate::arena::Arena;
use crate::arena::arena_impl::fixed_arena::FixedArena;
use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::TimedStep;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::interpreter::Interpreter;
use crate::ast::pattern::note::Frequency;
use crate::ast::pattern::note::Letter;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::player::pattern::MockPatternPlayer;
use crate::player::pattern::SoundUnit;

#[derive(Debug, Default)]
struct FixedArenas {
    pattern_arena: FixedArena<Pattern>,
    pattern_chain_arena: FixedArena<Chain<Pattern>>,
    timed_step_arena: FixedArena<TimedStep>,
    timed_step_chain_arena: FixedArena<Chain<TimedStep>>,
}

impl PatternArenas for FixedArenas {
    fn get_pattern_arena(&self) -> &impl Arena<Pattern> {
        &self.pattern_arena
    }

    fn get_pattern_chain_arena(&self) -> &impl Arena<Chain<Pattern>> {
        &self.pattern_chain_arena
    }

    fn get_timed_step_arena(&self) -> &impl Arena<TimedStep> {
        &self.timed_step_arena
    }

    fn get_timed_step_chain_arena(&self) -> &impl Arena<Chain<TimedStep>> {
        &self.timed_step_chain_arena
    }
}

#[test]
fn can_play_unit_for_one_cycle() {
    let note_unit = NoteUnit::Frequency(Frequency(440));
    let pattern_index = Index::new(0);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([(
            pattern_index.clone(),
            Pattern::Note(note_unit),
        )]),
        ..Default::default()
    };
    let mock_player = RefCell::new(MockPatternPlayer::new());
    let make_sound_unit = || SoundUnit::new(note_unit, CycleTime::ONE);
    let mut interpreter = Interpreter::new_with_refcell(
        pattern_index.clone(),
        &arenas,
        &mock_player,
    );
    // Test: Updating to one cycle will lead to exactly one invocation of
    // scheduling the note unit.
    mock_player
        .borrow_mut()
        .expect_schedule_note_unit()
        .with(predicate::eq(make_sound_unit()), predicate::eq(CycleTime::ZERO))
        .return_const(());
    interpreter.update_time(CycleTime::ONE);
    mock_player.borrow_mut().checkpoint();
    // Test: Updating the cycle to the same time does nothing.
    interpreter.update_time(CycleTime::ONE);
    mock_player.borrow_mut().checkpoint();
}

#[test]
fn can_play_singleton_cat_for_one_cycle() {
    // [ Cat ]
    //  ↓ ↓
    // [Unit(A)]
    let note_unit = NoteUnit::Letter(Letter::A);
    let pattern_index = Index::new(0);
    let arenas = FixedArenas {
        pattern_arena: FixedArena::new([(
            pattern_index.clone(),
            Pattern::Note(note_unit),
        )]),
        pattern_chain_arena: FixedArena::new([(
            Index::new(0xC0),
            Chain(pattern_index.clone(), None, None),
        )]),
        ..Default::default()
    };
    let mock_player = RefCell::new(MockPatternPlayer::new());
    let make_sound_unit = || SoundUnit::new(note_unit, CycleTime::ONE);
    let mut interpreter = Interpreter::new_with_refcell(
        pattern_index.clone(),
        &arenas,
        &mock_player,
    );
    // Test: Updating to one cycle will lead to exactly one invocation of
    // scheduling the note unit.
    let expect_note_unit = |start_time: CycleTime| {
        mock_player
            .borrow_mut()
            .expect_schedule_note_unit()
            .with(predicate::eq(make_sound_unit()), predicate::eq(start_time))
            .return_const(());
    };
    expect_note_unit(CycleTime::ZERO);
    interpreter.update_time(CycleTime::ONE);
    mock_player.borrow_mut().checkpoint();
    // Advancing by one cycle will give the same result.
    expect_note_unit(CycleTime::ONE);
    interpreter.update_time(CycleTime(CycleTime::ONE.0 + CycleTime::ONE.0));
    mock_player.borrow_mut().checkpoint();
}
