#![allow(dead_code, unused_variables, unused_imports)]
use std::default;

use bevy_ecs::prelude::*;
use super::{
    super::maths::{vector_three::Vector3, bounding_box::BoundingBox},
};

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Default, Component)]
pub struct PlantTag;

#[derive(Component)]
pub struct PlantData {
    pub position: Vector3,
    pub intersection_list: Vec<Entity>,
    pub age: f32,
    pub root_node: Option<Entity>,
    pub climate_adaption: f32,
}

#[derive(Component)]
pub struct PlantBounds {
    pub bounds: BoundingBox,
}


#[derive(Bundle)]
pub struct PlantBundle {
    pub tag: PlantTag,
    pub bounds: PlantBounds,
    pub data: PlantData,
    pub growth_factors: PlantGrowthControlFactors,
    pub plasticity_params: PlantPlasticityParameters,
}

#[derive(Resource)]
pub struct PlantDeathRate {
    pub v_max_decrease: f32,
}


#[derive(Component, Clone)]
pub struct PlantPlasticityParameters {
    pub seeding_frequency: f32,

    pub seeding_radius: f32,
    pub seeding_std_dev: f32,
    pub shadow_tolerance: f32,

}

#[derive(Clone)]
pub struct PlasticitySettingParams {
    pub seeding_frequency: f32,

    pub seeding_radius: f32,
    pub shadow_tolerance: f32,
}

impl Into<PlantPlasticityParameters> for PlasticitySettingParams {
    fn into(self) -> PlantPlasticityParameters {
        PlantPlasticityParameters {
            shadow_tolerance: self.shadow_tolerance.max(0.0000001),

            seeding_radius: self.seeding_radius.max(0.0000001),
            seeding_std_dev: self.seeding_radius.max(0.0000001) / 3.0, // ~99.7% of results fall into this area
            seeding_frequency: self.seeding_frequency.max(0.0000001),
        }
    }
}


#[derive(Component, Clone)]
pub struct PlantGrowthControlFactors {
    pub max_age: f32,

    pub species_max_vigor: f32,
    pub max_vigor: f32,
    pub min_vigor: f32,

    pub apical_control: f32, // range 0..1

    pub tropism_angle_weight: f32, // range 0..1

    pub growth_rate: f32,
    pub max_branch_segment_length: f32,
    pub branch_segment_length_scaling_coef: f32,
    pub tropism_time_control: f32,
    pub branching_angle: f32,
    pub thickening_factor: f32,
}

#[derive(Clone)]
pub struct GrowthControlSettingParams {
    pub max_age: f32,

    pub max_vigor: f32,
    pub min_vigor: f32,

    pub apical_control: f32, // range 0..1

    pub tropism_angle_weight: f32, // range 0..1

    pub growth_rate: f32,
    pub max_branch_segment_length: f32,
    pub branch_segment_length_scaling_coef: f32,
    pub tropism_time_control: f32,
    pub branching_angle: f32,
    pub thickening_factor: f32,
}

impl Into<PlantGrowthControlFactors> for GrowthControlSettingParams {
    fn into(self) -> PlantGrowthControlFactors {
        PlantGrowthControlFactors {
            species_max_vigor: self.max_vigor.max(0.0000001),
            max_vigor: self.max_vigor.max(0.0000001),
            min_vigor: self.min_vigor.max(0.0000001),

            max_age: self.max_age.max(0.0000001),

            apical_control: self.apical_control.clamp(0.0, 1.0),

            tropism_angle_weight: self.tropism_angle_weight.clamp(0.0, 1.0),
            growth_rate: self.growth_rate.max(0.0000001),
            max_branch_segment_length: self.max_branch_segment_length.max(0.0000001),
            branch_segment_length_scaling_coef: self.branch_segment_length_scaling_coef.max(0.0000001),

            tropism_time_control: self.tropism_time_control.max(0.0000001),
            branching_angle: self.branching_angle.max(0.0000001),
            thickening_factor: self.thickening_factor.max(0.0000001),
        }
    }
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


impl Default for PlantBundle {
    fn default() -> Self {
        PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::default(),
            data: PlantData::default(),
            growth_factors: PlantGrowthControlFactors::default(),
            plasticity_params: PlantPlasticityParameters::default(),
        }
    }
}

impl Default for PlantData {
    fn default() -> Self {
        PlantData {
            root_node: None,
            position: Vector3::ZERO(),
            intersection_list: Vec::new(),
            age: 0.0,
            climate_adaption: 1.0,
        }
    }
}

impl Default for PlantBounds {
    fn default() -> Self {
        PlantBounds {
            bounds: BoundingBox::ZERO(),
        }
    }
}

impl From<BoundingBox> for PlantBounds {
    fn from(bounds: BoundingBox) -> Self {
        Self {
            bounds
        }
    }
}

impl Default for PlantGrowthControlFactors {
    fn default() -> Self {
        PlantGrowthControlFactors {
            species_max_vigor: 0.0,
            max_vigor: 0.0,
            min_vigor: 0.0,

            max_age: 0.0,

            apical_control: 0.5,

            tropism_angle_weight: 0.5,
            growth_rate: 1.0,
            max_branch_segment_length: 1.0,
            branch_segment_length_scaling_coef: 1.0,

            tropism_time_control: 1.0,
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
            seeding_std_dev: 1.0 / 3.0,
            shadow_tolerance: 1.0,
        }
    }
}