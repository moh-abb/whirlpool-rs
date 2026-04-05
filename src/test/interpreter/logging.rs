#![allow(unused)]

use crate::ast::time::CycleTime;
use crate::player::PatternPlayer;
use crate::player::unit::SoundUnit;

pub struct LoggingPlayer<'a, P> {
    player: &'a mut P,
    log_enabled: bool,
}

impl<'a, P: PatternPlayer> LoggingPlayer<'a, P> {
    pub fn new(player: &'a mut P, log_enabled: bool) -> Self {
        Self { player, log_enabled }
    }

    pub fn get_mut_player(&mut self) -> &mut P {
        &mut self.player
    }
}

impl<'a, P: PatternPlayer> PatternPlayer for LoggingPlayer<'a, P> {
    fn schedule_note_unit(&mut self, sound: SoundUnit, start: CycleTime) {
        if self.log_enabled {
            println!("LoggingPlayer: Playing {sound:?} at {start:?}");
        }
        self.player
            .schedule_note_unit(sound, start);
    }
}
