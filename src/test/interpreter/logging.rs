#![allow(unused)]

use crate::ast::CycleTime;
use crate::synth::scheduler::UnitScheduler;
use crate::synth::unit::SoundUnit;

pub struct LoggingScheduler<'a, Scheduler> {
    scheduler: &'a mut Scheduler,
    inner_enabled: bool,
    log_enabled: bool,
}

impl<'a, Scheduler: UnitScheduler> LoggingScheduler<'a, Scheduler> {
    pub fn new(scheduler: &'a mut Scheduler) -> Self {
        Self { scheduler, inner_enabled: true, log_enabled: false }
    }

    pub fn set_inner_enabled(&mut self, inner_enabled: bool) {
        self.inner_enabled = inner_enabled;
    }

    pub fn set_logging(&mut self, log_enabled: bool) {
        self.log_enabled = log_enabled;
    }

    pub fn get_mut_scheduler(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }
}

impl<'a, P: UnitScheduler> UnitScheduler for LoggingScheduler<'a, P> {
    fn add(&mut self, sound: SoundUnit, start: CycleTime) {
        if !self.inner_enabled {
            return;
        }

        if self.log_enabled {
            println!("LoggingScheduler: Playing {sound:?} at {start:?}");
        }

        self.scheduler.add(sound, start);
    }
}
