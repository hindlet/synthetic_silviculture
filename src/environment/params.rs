//! this file is mainly to manage data about the entire environment, like gravity, temperature, and precipitation levels
#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use super::super::{
    maths::vector_three::Vector3
};


// gravity
pub struct GravityResources{
    pub gravity_dir: Vector3,
    pub tropism_strength: f32, // positive for gravitropism, negative for phototropism
}


// PhysicalAgeStep
pub struct PhysicalAgeStep{
    pub step: f32,
}
