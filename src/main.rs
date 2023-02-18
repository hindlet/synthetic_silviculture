#![allow(dead_code, unused_variables, unused_imports)]
use std::collections::HashMap;
use bevy_ecs::prelude::*;
use image::*;
use rand::*;

mod branch;
use branch::*;

mod branch_prototypes;
use branch_prototypes::*;

mod plant;
use plant::*;

mod tests;

mod general;
use general::*;

mod graphics;

mod branch_node;
use branch_node::*;

mod transform;
use transform::*;


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
