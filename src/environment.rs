//! this file is mainly to manage data about the entire environment, like gravity, temperature, and precipitation levels
#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use super::{
    maths::vector_three::Vector3
};


// gravity
#[derive(Resource)]
pub struct GravityResources{
    pub gravity_dir: Vector3,
    pub tropism_strength: f32, // positive for gravitropism, negative for phototropism
}

/// adds resouces for gravity into the world, normalises the direction of gravity
pub fn create_gravity_resource(
    world: &mut World,
    mut gravity_dir: Vector3,
    tropism_strength: f32
) {
    gravity_dir.normalise();

    world.insert_resource(GravityResources{
        gravity_dir,
        tropism_strength
    });
}


// PhysicalAgeStep
#[derive(Resource)]
pub struct PhysicalAgeStep{
    pub step: f32,
}

pub fn create_physical_age_time_step(
    world: &mut World,
    step: f32
) {
    world.insert_resource(PhysicalAgeStep{step});
}