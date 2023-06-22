#![allow(dead_code, unused_variables, unused_imports)]
use std::default;

use bevy_ecs::prelude::*;
use super::super::{
    maths::{vector_three::Vector3, bounding_box::BoundingBox},
    branches::{branch_node::{BranchNodeBundle, BranchNodeData}, branch::{BranchBundle, BranchData}, branch_prototypes::{BranchPrototypesSampler, BranchPrototypeRef}},
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


#[derive(Component, Clone, PartialEq)]
pub struct PlantPlasticityParameters {
    pub flowering_age: f32,
    pub seeding_frequency: f32,
    pub seeding_interval: f32,
    pub time_since_seeding: f32,
    pub is_seeding: bool,

    pub seeding_radius: f32,
    pub seeding_std_dev: f32,

    pub shadow_tolerance: f32,
}

#[derive(Clone)]
pub struct PlasticitySettingParams {
    pub seeding_frequency: f32,
    pub flowering_age: f32,

    pub seeding_radius: f32,
    pub shadow_tolerance: f32,
}

impl PlantPlasticityParameters {
    pub fn copy_for_new_plant(&self) -> PlantPlasticityParameters {
        PlantPlasticityParameters {
            shadow_tolerance: self.shadow_tolerance,

            seeding_radius: self.seeding_radius,
            seeding_std_dev: self.seeding_std_dev, // ~99.7% of results fall into this area
            seeding_frequency: self.seeding_frequency,
            seeding_interval: self.seeding_interval,
            time_since_seeding: 0.0,
            is_seeding: false,
            flowering_age: self.flowering_age,
        }
    }
}

impl PlasticitySettingParams {
    pub fn into_plasticity(self, time_step: f32) -> PlantPlasticityParameters {
        PlantPlasticityParameters {
            shadow_tolerance: self.shadow_tolerance.max(0.0000001),

            seeding_radius: self.seeding_radius.max(0.0000001),
            seeding_std_dev: self.seeding_radius.max(0.0000001) / 3.0, // ~99.7% of results fall into this area
            seeding_frequency: self.seeding_frequency.max(0.0000001),
            seeding_interval: 1.0 / self.seeding_frequency.max(0.0000001),
            time_since_seeding: 0.0,
            is_seeding: false,
            flowering_age: self.flowering_age,
        }
    }
}


#[derive(Component, Clone, PartialEq)]
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

impl PlantGrowthControlFactors {
    pub fn copy_for_new_plant(&self) -> PlantGrowthControlFactors {
        PlantGrowthControlFactors {
            species_max_vigor: self.species_max_vigor,
            max_vigor: self.species_max_vigor,
            min_vigor: self.min_vigor,

            max_age: self.max_age,

            apical_control: self.apical_control,

            tropism_angle_weight: self.tropism_angle_weight,
            growth_rate: self.growth_rate,
            max_branch_segment_length: self.max_branch_segment_length,
            branch_segment_length_scaling_coef: self.branch_segment_length_scaling_coef,

            tropism_time_control: self.tropism_time_control,
            branching_angle: self.branching_angle,
            thickening_factor: self.thickening_factor,
        }
    }
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
/////////////////////////////////////// fn ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


pub fn spawn_plant(
    root_position: Vector3,
    normal: Vector3,

    growth_settings: PlantGrowthControlFactors,
    plasticity_settings: PlantPlasticityParameters,
    plant_climate_adaptation: f32,

    branch_sampler: &BranchPrototypesSampler,

    commands: &mut Commands,
) -> (Entity, Entity, Entity){
    let root_node_id = commands.spawn(BranchNodeBundle{
        data: BranchNodeData{
            thickening_factor: growth_settings.thickening_factor,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let root_branch_id = commands.spawn(BranchBundle{
        data: BranchData {
            root_node: Some(root_node_id),
            root_position: root_position,
            ..Default::default()
        },
        prototype: BranchPrototypeRef(branch_sampler.get_prototype_index(growth_settings.apical_control, branch_sampler.max_determinancy)),
        ..Default::default()
    }).id();

    let plant_id = commands.spawn(PlantBundle{
        growth_factors: growth_settings,
        data: PlantData {
            root_node: Some(root_branch_id),
            position: root_position,
            climate_adaption: plant_climate_adaptation,
            ..Default::default()
        },
        plasticity_params: plasticity_settings,
        ..Default::default()
    }).id();

    (root_branch_id, root_branch_id, plant_id)
}

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
            seeding_interval: 2.0,
            time_since_seeding: 0.0,
            is_seeding: false,
            flowering_age: 0.0,

            seeding_radius: 1.0,
            seeding_std_dev: 1.0 / 3.0,

            shadow_tolerance: 1.0,
        }
    }
}