//! A wrapper for a scedule so it runs on a fixed timescale
//! 
//! It was heavily based off of the bevy game engine fixed update timer

use bevy_ecs::prelude::*;
use std::time::Duration;
use std::error::Error;

pub struct FixedSchedule {
    accumulated: Duration,
    pub period: Duration,
    pub schedule: Schedule,
}

impl FixedSchedule {

    pub fn new(period: Duration, schedule: Schedule) -> Self{
        FixedSchedule {
            accumulated: Duration::ZERO,
            period,
            schedule
        }
    }

    pub fn tick(&mut self, delta_time: Duration) {
        self.accumulated += delta_time;
    }

    pub fn expend(&mut self) -> Result<(), ()> {
        if let Some(new_value) = self.accumulated.checked_sub(self.period) {
            self.accumulated = new_value;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn run(&mut self, world: &mut World, delta_time: Duration) {
        self.tick(delta_time);

        // run as many times as required since the last run
        let mut check_again = true;
        while check_again {
            if self.expend().is_ok() {
                self.schedule.run(world);
            } else {
                check_again = false;
            }
        }
    }
}

