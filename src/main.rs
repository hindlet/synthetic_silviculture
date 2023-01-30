#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use image::*;
use rand::*;
mod general;
use general::*;

mod branch;
use branch::*;

mod branch_prototypes;
use branch_prototypes::*;

mod plant;
use plant::*;

mod tests;


fn main() {

    // Create a new empty World to hold our Entities and Components
    let mut world = World::new();

    #[derive(StageLabel)]
    pub struct SetupLabel;

    let mut startup_schedule = Schedule::default();

    startup_schedule.add_stage(SetupLabel, SystemStage::parallel());
    // startup_schedule.add_system_to_stage(SetupLabel, create_branch_prototypes);

    startup_schedule.run(&mut world);

    
}