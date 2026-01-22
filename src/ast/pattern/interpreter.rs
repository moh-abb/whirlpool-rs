use core::cell::RefCell;
use core::ops::DerefMut;

use fixed::FixedU32;
use fixed::Wrapping;

use crate::arena::chain::Chain;
use crate::arena::index::Index;
use crate::ast::multiple::Multiple;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::arenas::PatternArenas;
use crate::ast::pattern::note::NoteUnit;
use crate::ast::pattern::visitor::PatternVisitor;
use crate::ast::pattern::visitor::visit_pattern;
use crate::ast::time::CycleTime;
use crate::player::pattern::PatternPlayer;
use crate::player::pattern::SoundUnit;

#[allow(unused)]
pub struct Interpreter<'a, Arenas, Player> {
    pattern: Index<Pattern>,
    arenas: &'a Arenas,
    player: &'a RefCell<Player>,
    position: CycleTime,
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer>
    Interpreter<'a, Arenas, Player>
{
    #[allow(unused)]
    pub fn new(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        player: &'a RefCell<Player>,
    ) -> Self {
        Self { pattern, arenas, player, position: CycleTime(Wrapping::ZERO) }
    }

    #[allow(unused)]
    pub fn update_time(&mut self, next_position: CycleTime) {
        // The time should be monotonically increasing.
        assert!(next_position >= self.position);
        let mut borrowed_player = self.player.borrow_mut();
        let visitor = InterpreterVisitor {
            arenas: self.arenas,
            start: self.position,
            end: next_position,
            multiplier: 1,
            inner: RefCell::new(VisitorInner {
                player: borrowed_player.deref_mut(),
            }),
        };
        visit_pattern(&visitor, self.pattern.clone());
        self.position = next_position;
    }
}

struct InterpreterVisitor<'a, Arenas, Player> {
    arenas: &'a Arenas,
    start: CycleTime,
    end: CycleTime,
    multiplier: u32,
    inner: RefCell<VisitorInner<'a, Player>>,
}

struct VisitorInner<'a, Player> {
    player: &'a mut Player,
}

#[allow(unused)]
#[derive(Debug, Clone)]
enum MultipleType {
    Cat { length: u16 },
    Seq { length: u16 },
    Stack { length: u16 },
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer> PatternVisitor
    for InterpreterVisitor<'a, Arenas, Player>
{
    type Output = ();
    type PatternOutput = ();
    type PatternChainOutput = MultipleType;
    type TimedStepOutput = ();
    type TimedStepChainOutput = ();

    const CHAINS_FOLD_RIGHT: bool = false;

    fn get_arenas(&self) -> &impl PatternArenas {
        self.arenas
    }

    fn map_pattern(
        &self,
        _pattern_index: Index<Pattern>,
        pattern_output: Self::PatternOutput,
    ) -> Self::Output {
        pattern_output
    }

    fn map_cat(
        &self,
        _multiple: Multiple<Pattern>,
        _pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_seq(
        &self,
        _multiple: Multiple<Pattern>,
        _pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_stack(
        &self,
        _multiple: Multiple<Pattern>,
        _pattern_chain_output: Self::PatternChainOutput,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_time_cat(
        &self,
        _multiple: Multiple<super::TimedStep>,
        _timed_step_chain_output: Self::TimedStepChainOutput,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let mut inner = self.inner.borrow_mut();
        let multiplier = Wrapping(FixedU32::const_from_int(self.multiplier));
        let _ = multiplier.frac();
        let start = self.start.0;
        let end = self.end.0;
        let recip_multiplier = multiplier.recip();
        let scaled_start = (start * multiplier).ceil();
        let scaled_end = (end * multiplier).ceil();
        let mut cur = scaled_start;
        let sound_unit = SoundUnit::new(unit, CycleTime(recip_multiplier));
        while cur < scaled_end {
            let note_unit_start = cur * recip_multiplier;
            inner.player.schedule_note_unit(
                sound_unit.clone(),
                CycleTime(note_unit_start),
            );
            cur += Wrapping::from_num(1);
        }
    }

    fn map_silence(&self) -> Self::PatternOutput {
        todo!()
    }

    fn new_pattern_chain_output(&self) -> Self::PatternChainOutput {
        todo!()
    }

    fn fold_pattern_chain_output(
        &self,
        _multiple_type: Self::PatternChainOutput,
        _tail_index: Index<Chain<Pattern>>,
        _pattern_index: Index<Pattern>,
    ) -> Self::PatternChainOutput {
        todo!()
    }

    fn new_timed_step_chain_output(&self) -> Self::TimedStepChainOutput {}

    fn fold_timed_step_chain_output(
        &self,
        _timed_step_chain_output: Self::TimedStepChainOutput,
        _tail_index: Index<Chain<super::TimedStep>>,
        _timed_step_index: Index<super::TimedStep>,
    ) -> Self::TimedStepChainOutput {
    }
}
