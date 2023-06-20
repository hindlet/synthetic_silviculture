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
const DEFAULT_BRANCH_MESH_SETTINGS: (u32, bool) = (3, false);
const DEFAULT_ENVIRONMENTAL_PARAMS: (f32, f32, f32) = (10.0, 0.01, 110.0); // based on the UK
const DEFAULT_TERRAIN: (f32, [f32; 3]) = (50.0, [0.0, 0.0, 0.0]);

const DEFAULT_BRANCH_TYPES: Vec<(f32, Vec<Vec<u32>>, Vec<[f32; 3]>)> = Vec::new();
const DEFAULT_BRANCH_CONTIDITIONS: (Vec<(f32, f32)>, f32, f32) = (Vec::new(), 1.0, 1.0);
const DEFAULT_PLANT_SPECIES: Vec<((GrowthControlSettingParams, PlasticitySettingParams), (f32, f32, f32, f32))> = Vec::new();


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
    world: &mut World,
) -> Vec<([f32; 3], Vec<([f32; 3], f32)>, Vec<(usize, usize)>)>{

    let mut state: SystemState<(
        Query<&BranchNodeData, With<BranchNodeTag>>,
        Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
        Query<&BranchData, With<BranchTag>>,
        Query<&BranchConnectionData, With<BranchTag>>,
        Query<&PlantData, With<PlantBounds>>,
    )> = SystemState::new(world);

    let (node_data, node_connections, branch_data, branch_connections, plant_data) = state.get(world);

    // plant positions : node position and thickness : node connections
    let mut data: Vec<([f32; 3], Vec<([f32; 3], f32)>, Vec<(usize, usize)>)> = Vec::new();
    for plant in plant_data.iter() {
        if plant.root_node.is_none() {continue;}
        
        let position: [f32; 3] = plant.position.into();

        let mut plant_data: (Vec<([f32; 3], f32)>, Vec<(usize, usize)>) = (Vec::new(), Vec::new());

        let mut current_nodes: usize = 0;

        for id in get_branches_base_to_tip(&branch_connections, plant.root_node.unwrap()) {
            if let Ok(branch) = branch_data.get(id) {
                if branch.root_node.is_none() {continue;}

                let (positions, thicknesses, pairs) = get_node_data_and_connections_base_to_tip(&node_connections, &node_data, branch.root_node.unwrap());
                let mut sub_data: Vec<([f32; 3], f32)> = Vec::new();
                for i in 0..positions.len() {
                    sub_data.push((positions[i].into(), thicknesses[i]));
                }
                let start_point = plant_data.0.iter().position(|&x| x == sub_data[0]).unwrap_or(0);
                if start_point != 0 {current_nodes -= 1; sub_data.remove(0);}

                let mut new_pairs: Vec<(usize, usize)> = Vec::new();
                for pair in pairs {
                    let one = if pair.0 == 0 {start_point} else {pair.0 + current_nodes};
                    let two = if pair.0 == 0 {start_point} else {pair.0 + current_nodes};
                    new_pairs.push((one, two));
                }
                plant_data.0.append(&mut sub_data);
                plant_data.1.append(&mut new_pairs);
                current_nodes = plant_data.0.len() - 1;
            }
        }
        data.push((position, plant_data.0, plant_data.1));
    }


    data
}