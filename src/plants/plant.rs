#![allow(dead_code, unused_variables, unused_imports)]
use std::default;

use bevy_ecs::prelude::*;

use crate::branches::branch_sorting::get_branches_base_to_tip;

use super::super::{
    maths::{vector_three::Vector3, bounding_box::BoundingBox},
    branches::branch::*,
};

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Default, Component)]
pub struct PlantTag;


pub struct Plant {
    pub position: Vector3,
    pub age: f32,
    pub root: Branch,

    pub growth_factors: PlantGrowthControlFactors,
    pub plasticity: PlantPlasticityParameters,
    pub bounds: BoundingBox
}



#[derive(Component)]
pub struct PlantData {
    pub position: Vector3,
    pub age: f32,
    pub root_branch: Branch,
}

#[derive(Resource)]
pub struct PlantDeathRate {
    pub v_max_decrease: f32,
}


#[derive(Component, Clone)]
pub struct PlantPlasticityParameters {
    pub seeding_frequency: f32,
    pub seeding_radius: f32,
    pub shadow_tolerance: f32,
}


#[derive(Component, Clone)]
pub struct PlantGrowthControlFactors {
    pub max_age: f32,
    pub max_vigor: f32,
    pub min_vigor: f32,
    pub apical_control: f32, // range 0..1
    pub tropism_angle_weight: f32, // range 0..1
    pub growth_rate: f32,
    pub max_branch_segment_length: f32,
    pub branch_segment_length_scaling_coef: f32,
    pub tropism_control: f32,
    pub branching_angle: f32,
    pub thickening_factor: f32,
}


///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////// impls /////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

impl PlantDeathRate {
    pub fn new(death_rate: f32) -> Self {
        PlantDeathRate {
            v_max_decrease: death_rate,
        }
    }
}


impl Default for PlantData {
    fn default() -> Self {
        PlantData {
            root_branch: None,
            position: Vector3::ZERO(),
            age: 0.0,
        }
    }
}



impl Default for PlantGrowthControlFactors {
    fn default() -> Self {
        PlantGrowthControlFactors {
            max_vigor: 0.0,
            min_vigor: 0.0,
            max_age: 0.0,
            apical_control: 0.5,
            tropism_angle_weight: 0.5,
            growth_rate: 1.0,
            max_branch_segment_length: 1.0,
            branch_segment_length_scaling_coef: 1.0,
            tropism_control: 1.0,
            branching_angle: 0.0,
            thickening_factor: 0.01,
        }
    }
}   


impl Default for PlantPlasticityParameters {
    fn default() -> Self {
        PlantPlasticityParameters {
            seeding_frequency: 0.5,
            seeding_radius: 1.0,
            shadow_tolerance: 1.0,
        }
    }
}