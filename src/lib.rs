#![allow(unused_imports)]

pub mod branch;
mod branch_prototypes;
pub mod general;
pub mod plant;
pub mod branch_node;
pub mod graphics;

use bevy_ecs::schedule::{SystemSet, Schedule};
use branch::*;
use branch_prototypes::*;
use plant::*;
use general::*;


mod tests;


use branch_node::*;

mod transform;


pub fn get_simulation_schedule() -> Schedule {
    let schedule = Schedule::default();

    


    schedule
}