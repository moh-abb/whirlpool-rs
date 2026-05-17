#![allow(unused)]

use crate::ast::CycleTime;
use crate::player::PatternPlayer;
use crate::player::unit::SoundUnit;

pub struct LoggingPlayer<'a, P> {
    player: &'a mut P,
    inner_enabled: bool,
    log_enabled: bool,
}

impl<'a, P: PatternPlayer> LoggingPlayer<'a, P> {
    pub fn new(player: &'a mut P) -> Self {
        Self { player, inner_enabled: true, log_enabled: false }
    }

    pub fn set_inner_enabled(&mut self, inner_enabled: bool) {
        self.inner_enabled = inner_enabled;
    }

    pub fn set_logging(&mut self, log_enabled: bool) {
        self.log_enabled = log_enabled;
    }

    pub fn get_mut_player(&mut self) -> &mut P {
        &mut self.player
    }
}

impl<'a, P: PatternPlayer> PatternPlayer for LoggingPlayer<'a, P> {
    fn schedule_note_unit(&mut self, sound: SoundUnit, start: CycleTime) {
        if !self.inner_enabled {
            return;
        }

        if self.log_enabled {
            println!("LoggingPlayer: Playing {sound:?} at {start:?}");
        }

        self.player
            .schedule_note_unit(sound, start);
    }
}
