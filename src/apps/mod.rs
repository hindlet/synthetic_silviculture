use std::ops::RangeInclusive;
use bevy_ecs::{
    prelude::*,
    system::SystemState
};
use rand::{Rng, thread_rng};
use crate::maths::colliders::mesh_collider::MeshCollider;
use super::{
    branches::{
        branch::*,
        branch_development::*,
        branch_prototypes::*,
        branch_node::*,
        branch_sorting::*,
        node_sorting::*,
    },
    plants::{
        plant::*,
        plant_development::*,
    },
    maths::{
        vector_three::Vector3,
        colliders::Collider,
    },
};

#[cfg(feature = "vulkan_graphics")]
pub mod graphics_app;
pub mod looped_app;


//////////////////// consts
const SAMPLER_SIZE: (u32, u32) = (500, 500);

const DEFAULT_GRAVITY_STRENGTH: f32 = 0.5;
const DEFAULT_TIMESTEP: f32 = 1.0;
const DEFAULT_CELL_SETTINGS: (u32, f32) = (5, 0.5);
const DEFAULT_PLANT_DEATH_RATE: f32 = 1.0;
const DEFAULT_LIGHT: ([f32; 3], f32) = ([0.0, -1.0, 0.0], 0.5);
const DEFAULT_ENVIRONMENTAL_PARAMS: (f32, f32, f32) = (10.0, 0.01, 110.0); // based on the UK
const DEFAULT_TERRAIN: (f32, [f32; 3]) = (50.0, [0.0, 0.0, 0.0]);

const DEFAULT_BRANCH_TYPES: Vec<(f32, Vec<Vec<u32>>, Vec<[f32; 3]>)> = Vec::new();
const DEFAULT_BRANCH_CONTIDITIONS: (Vec<(f32, f32)>, f32, f32) = (Vec::new(), 1.0, 1.0);
const DEFAULT_PLANT_SPECIES: Vec<((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))> = Vec::new();

enum TerrainType {
    Absent,
    Flat,
    Bumpy
}

enum OutputType {
    Absent,
    Data,
    Meshes,
    All,
}



#[derive(Clone)]
pub struct TreeAppOutput {
    pub data: Option<Vec<([f32; 3], Vec<([f32; 3], f32)>, Vec<(usize, usize)>)>>,
    pub meshes: Option<()>,
}

impl Default for TreeAppOutput {
    fn default() -> Self {
        TreeAppOutput {data: None, meshes: None}
    }
}


fn data_output(
    plants: &Vec<Plant>
) -> Vec<(Vec<([f32; 3], f32)>, Vec<(usize, usize)>)>{

    let mut out = Vec::new();

    for plant in plants.iter() {
        let plant_pos = plant.position;

        let mut node_data: Vec<([f32; 3], f32)> = Vec::new();
        let mut node_pairs: Vec<(usize, usize)> = Vec::new();

        let mut node_offset = 0;

        for branch in get_branches_base_to_tip(&plant.root) {

            let (intial_data, initial_pairs) = get_node_data_and_connections_base_to_tip(&branch.root);
        
            // reposition nodes
            let mut final_data: Vec<([f32; 3], f32)> = Vec::new();
            for data in intial_data {
                final_data.push(((data.0 + branch.data.root_position).into(), data.1));
            }


            // find the index of the starting node  
            let zero_index = node_data.iter().position(|&x| x == final_data[0]).unwrap_or(0);
            if zero_index != 0 {
                node_offset -= 1;
                final_data.remove(0);
            }

            let mut adjusted_pairs = Vec::new();
            for pair in initial_pairs {
                let one = if pair.0 == 0 {zero_index} else {pair.0 + node_offset};
                adjusted_pairs.push((one, pair.1 + node_offset));
            }

            node_data.append(&mut final_data);
            node_pairs.append(&mut adjusted_pairs);

            node_offset = node_data.len() - 1;
        }

        out.push((node_data, node_pairs));

    }

    out
}