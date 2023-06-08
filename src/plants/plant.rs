#![allow(dead_code, unused_variables, unused_imports)]
use std::{default, rc::Rc, cell::RefCell};
use super::super::{
    maths::{vector_three::Vector3, bounding_box::BoundingBox},
    branches::branch::*,
};

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub struct Plant {
    pub position: Vector3,
    pub age: f32,
    pub root: Rc<RefCell<Branch>>,

    pub growth_factors: PlantGrowthControlFactors,
    pub plasticity: PlantPlasticityParameters,
    pub bounds: BoundingBox
}


#[derive(Clone)]
pub struct PlantPlasticityParameters {
    pub seeding_frequency: f32,
    pub seeding_radius: f32,
    pub shadow_tolerance: f32,
}


#[derive(Clone)]
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

impl Plant {
    pub fn new(
        plasticity: PlantPlasticityParameters,
        growth_factors: PlantGrowthControlFactors,
        position: impl Into<Vector3>,
        normal: impl Into<Vector3>,
        prototype_id: usize
    ) -> Self {

        let thick = growth_factors.thickening_factor;
        Plant {
            plasticity: plasticity,
            position: position.into(),
            age: 0.0,
            growth_factors: growth_factors,
            bounds: BoundingBox::ZERO(),
            root: Rc::new(RefCell::new(Branch::new(Vector3::ZERO(), thick, normal.into(), prototype_id, 0, 0))),
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