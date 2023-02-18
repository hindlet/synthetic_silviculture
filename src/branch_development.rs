#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::{
    plant::{PlantData, PlantGrowthControlFactors, PlantTag},
    branch::{BranchGrowthData, BranchTag, BranchConnectionData, get_branches_base_to_tip, get_terminal_branches, BranchData},
    branch_node::{BranchNodeGrowthData, BranchNodeConnectionData, BranchNodeTag, get_terminal_nodes, get_nodes_tip_to_base, get_nodes_base_to_tip},
    branch_prototypes::{BranchPrototypeData, BranchPrototypeRef},
};

/// calculates growth rates for all branches: 
/// Growth Rate = Sigmoid( (branch_vigor - plant_min_vigor)/(plant_max_vigor - plant_min_vigor) ) * plant_growth_rate
pub fn assign_growth_rates (
    plant_query: Query<(&PlantData, &PlantGrowthControlFactors), With<PlantTag>>,
    mut branch_data_query: Query<&mut BranchGrowthData, With<BranchTag>>,
    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
) {
    for (plant_data, plant_growth_factors) in plant_query.iter() {

        if plant_data.root_node.is_none() {continue;}
        let v_min = plant_growth_factors.min_vigor;
        let v_max = plant_growth_factors.max_vigor;
        let plant_growth = plant_growth_factors.growth_rate;
        
        for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {
            if let Ok(mut branch_data) = branch_data_query.get_mut(id){
                branch_data.growth_rate = sigmoid((branch_data.growth_vigor - v_min) / (v_max - v_min)) * plant_growth;
            }
        }
    }
}

fn sigmoid(x: f32) -> f32 {
    3.0 * x.powi(2) - 2.0 * x.powi(3)
}

/// decides if and where to create new branches on a plant
pub fn determine_create_new_branches(
    plant_query: Query<(&PlantData, &PlantGrowthControlFactors), With<PlantTag>>,
    branch_data_query: Query<(&BranchData, &BranchGrowthData, &BranchPrototypeRef), With<BranchTag>>,
    connections_query: Query<&BranchConnectionData, With<BranchTag>>,
    branch_prototype_query: Query<&BranchPrototypeData>,
    mut node_data_query: Query<&mut BranchNodeGrowthData, With<BranchNodeTag>>,
    node_connection_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>
) {
    for (plant_data, plant_growth_factors) in plant_query.iter() {
        if plant_data.root_node.is_none() {continue;}

        let branches_to_check = get_terminal_branches(&connections_query, plant_data.root_node.unwrap());
        let v_min = plant_growth_factors.min_vigor;

        for id in branches_to_check {
            if let Ok((branch_data, branch_growth_data, prototype_ref)) = branch_data_query.get(id) {
                if branch_data.root_node.is_none() {continue;}
                // branch must be older than mature for new branches to be added
                if let Ok(prototype) = branch_prototype_query.get(prototype_ref.0) {
                    if branch_growth_data.physiological_age <= prototype.mature_age {continue;}
                }

                let terminal_node_ids = get_terminal_nodes(&node_connection_query, branch_data.root_node.unwrap());
                let node_light_level = branch_growth_data.light_exposure / terminal_node_ids.len() as f32;
                for id in terminal_node_ids {
                    if let Ok(mut node_data) = node_data_query.get_mut(id) {
                        node_data.light_exposure = node_light_level;
                    }
                }

                // sum up light exposure at branching points
                for id in get_nodes_tip_to_base(&node_connection_query, branch_data.root_node.unwrap()) {
                    #[allow(unused_assignments)]
                    let mut light_exposure = 0.0;
                    if let Ok(node_data) = node_data_query.get(id) {
                        light_exposure = node_data.light_exposure;
                    } else {panic!("Fuck shit balls")}
                    if let Ok(node_connections) = node_connection_query.get(id) {
                        if node_connections.parent.is_none() {continue;}
                        if let Ok(mut parent_data) = node_data_query.get_mut(node_connections.parent.unwrap()) {
                            parent_data.light_exposure += light_exposure;
                        }
                    }
                }


            }
        }
    }
}