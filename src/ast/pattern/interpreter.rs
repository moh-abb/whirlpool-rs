use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::DerefMut;

use fixed::FixedU32;
use fixed::Wrapping;

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

mod private {
    use core::cell::RefCell;
    use core::ops::DerefMut;

    pub trait BorrowAdapter<T> {
        fn borrow_mut(&mut self) -> impl DerefMut<Target = T>;
    }

    impl<T> BorrowAdapter<T> for &RefCell<T> {
        fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
            RefCell::borrow_mut(self)
        }
    }

    impl<T> BorrowAdapter<T> for &mut T {
        fn borrow_mut(&mut self) -> impl DerefMut<Target = T> {
            self.deref_mut()
        }
    }
}

#[allow(unused)]
pub struct Interpreter<'a, Arenas, Player, B: private::BorrowAdapter<Player>> {
    pattern: Index<Pattern>,
    arenas: &'a Arenas,
    borrow_adapter: B,
    position: CycleTime,
    phantom: PhantomData<Player>,
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer>
    Interpreter<'a, Arenas, Player, &'a mut Player>
{
    #[allow(unused)]
    pub fn new(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        player: &'a mut Player,
    ) -> Self {
        Self {
            pattern,
            arenas,
            borrow_adapter: player,
            position: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<'a, Arenas: PatternArenas, Player: PatternPlayer>
    Interpreter<'a, Arenas, Player, &'a RefCell<Player>>
{
    #[allow(unused)]
    pub fn new_with_refcell(
        pattern: Index<Pattern>,
        arenas: &'a Arenas,
        player: &'a RefCell<Player>,
    ) -> Self {
        Self {
            pattern,
            arenas,
            borrow_adapter: player,
            position: CycleTime::ZERO,
            phantom: PhantomData,
        }
    }
}

impl<
    'a,
    Arenas: PatternArenas,
    Player: PatternPlayer,
    Borrow: private::BorrowAdapter<Player>,
> Interpreter<'a, Arenas, Player, Borrow>
{
    #[allow(unused)]
    pub fn update_time(&mut self, next_position: CycleTime) {
        // The time should be monotonically increasing.
        assert!(next_position >= self.position);
        let mut borrowed_player = self.borrow_adapter.borrow_mut();
        let visitor = InterpreterVisitor {
            arenas: self.arenas,
            start: self.position,
            end: next_position,
            multiplier: 1,
            inner: RefCell::new(VisitorInner {
                player: borrowed_player.deref_mut(),
                _opt_multiple_type: None,
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

impl<'a, Arenas, Player> InterpreterVisitor<'a, Arenas, Player> {
    fn play_multiple(&self, mut play_elem: impl FnMut(CycleTime)) {
        let multiplier = CycleTime(Wrapping::from_num(self.multiplier));
        let recip_multiplier = multiplier.0.recip();
        let scaled_start = CycleTime((self.start.0 * multiplier.0).ceil());
        let scaled_end = CycleTime((self.end.0 * multiplier.0).ceil());
        let mut cur = scaled_start;
        while cur.0 < scaled_end.0 {
            play_elem(CycleTime(cur.0 * recip_multiplier));
            cur.0 += CycleTime::ONE.0;
        }
    }
}

struct VisitorInner<'a, Player> {
    player: &'a mut Player,
    _opt_multiple_type: Option<MultipleType>,
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

    fn map_cat(&self, _multiple: Multiple<Pattern>) -> Self::PatternOutput {}

    fn map_seq(&self, _multiple: Multiple<Pattern>) -> Self::PatternOutput {
        todo!()
    }

    fn map_stack(&self, _multiple: Multiple<Pattern>) -> Self::PatternOutput {
        todo!()
    }

    fn map_time_cat(
        &self,
        _multiple: Multiple<super::TimedStep>,
    ) -> Self::PatternOutput {
        todo!()
    }

    fn map_note_unit(&self, unit: NoteUnit) -> Self::PatternOutput {
        let mut inner = self.inner.borrow_mut();
        let multiplier = Wrapping(FixedU32::const_from_int(self.multiplier));
        let unit_duration = CycleTime(multiplier.recip());
        let sound_unit = SoundUnit::new(unit, unit_duration);
        self.play_multiple(|start| {
            inner
                .player
                .schedule_note_unit(sound_unit.clone(), start);
        });
    }

    fn map_silence(&self) -> Self::PatternOutput {
        todo!()
    }
}
