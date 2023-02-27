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
    node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>
) {
    for (plant_data, plant_growth_factors) in plant_query.iter() {
        if plant_data.root_node.is_none() {continue;}

        let branches_to_check = get_terminal_branches(&connections_query, plant_data.root_node.unwrap());
        let v_min = plant_growth_factors.min_vigor;

        // loop through every terminal branch
        for id in branches_to_check {
            if let Ok((branch_data, branch_growth_data, prototype_ref)) = branch_data_query.get(id) {
                
                if branch_data.root_node.is_none() {continue;}
                // branch must be older than mature for new branches to be added
                if let Ok(prototype) = branch_prototype_query.get(prototype_ref.0) {
                    if branch_growth_data.physiological_age <= prototype.mature_age {continue;}
                }

                // assign light exposure to terminal branch nodes
                let terminal_node_ids = get_terminal_nodes(&node_connections_query, branch_data.root_node.unwrap());
                let node_light_level = branch_growth_data.light_exposure / terminal_node_ids.len() as f32;
                for id in terminal_node_ids {
                    if let Ok(mut node_data) = node_data_query.get_mut(id) {
                        node_data.light_exposure = node_light_level;
                    }
                }

                // get the nodes on the branch that could generate new branches
                let possible_terminal_nodes = get_possible_new_branch_nodes(&mut node_data_query, &node_connections_query, branch_data.root_node.unwrap(), v_min);

            }
        }
    }
}


/// helper func to decide which nodes in a branch could generate a new branch
/// returns the two nodes on the branch that are the options for branch attachments
fn get_possible_new_branch_nodes (
    node_data_query: &mut Query<&mut BranchNodeGrowthData, With<BranchNodeTag>>,
    node_connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
    branch_min_vigor: f32,
) -> (Option<Entity>, Option<Entity>) {
    

    // sum up light exposure at branching points
    for id in get_nodes_tip_to_base(&node_connections_query, root_node) {
        #[allow(unused_assignments)]
        let mut light_exposure = 0.0;
        if let Ok(node_data) = node_data_query.get(id) {
            light_exposure = node_data.light_exposure;
        } else {panic!("Fuck shit balls")}
        if let Ok(node_connections) = node_connections_query.get(id) {
            if node_connections.parent.is_none() {continue;}
            if let Ok(mut parent_data) = node_data_query.get_mut(node_connections.parent.unwrap()) {
                parent_data.light_exposure += light_exposure;
            }
        }
    }

    if let Ok(mut root_node_data) = node_data_query.get_mut(root_node) {
        root_node_data.growth_vigor = root_node_data.light_exposure;
    }

    // distribute vigor up the branch, there is no apical control so it's slightly easier
    for id in get_nodes_base_to_tip(&node_connections_query, root_node) {
        #[allow(unused_assignments)]
        let mut parent_vigor = 0.0;        

        if let Ok(parent_data) = node_data_query.get(id) {
            parent_vigor = parent_data.growth_vigor;
        } else {panic!("Oh god oh no the end is nigh")}

        if let Ok(parent_connections) = node_connections_query.get(id) {

            if parent_connections.children.len() == 0 {continue;}
            if parent_connections.children.len() == 1 {
                if let Ok(mut only_child) = node_data_query.get_mut(parent_connections.children[0]) {
                    only_child.growth_vigor = parent_vigor;
                    continue;
                }
            }

            // calculate the sum of all the child light exposures
            let child_light_sum = {
                let mut light_sum = 0.0;

                for id in parent_connections.children.iter() {
                    if let Ok(child_data) = node_data_query.get(*id) {
                        light_sum += child_data.light_exposure;
                    }
                }
                light_sum
            };

            // distribute parent vigor to children
            for id in parent_connections.children.iter() {
                if let Ok(mut child_data) = node_data_query.get_mut(*id) {
                    child_data.growth_vigor = parent_vigor * (child_data.light_exposure/child_light_sum);
                }
            }
        }
    }

    
    // get the possible nodes
    let mut possible_nodes = Vec::new();

    for id in get_terminal_nodes(&node_connections_query, root_node) {
        if let Ok(node_data) = node_data_query.get(id) {
            if node_data.growth_vigor > branch_min_vigor {possible_nodes.push((id, node_data.growth_vigor))}
        }
    }

    // check cases for 0 or 1 node
    if possible_nodes.len() == 0 {return (None, None);}
    if possible_nodes.len() == 1 {return (Some(possible_nodes[0].0), None);}

    // create the inital best nodes
    let mut best_nodes = 
        if possible_nodes[0].1 > possible_nodes[1].1 {
            ((possible_nodes[0].0, possible_nodes[0].1), (possible_nodes[1].0, possible_nodes[1].1))
        }
        else {
            ((possible_nodes[1].0, possible_nodes[1].1), (possible_nodes[0].0, possible_nodes[0].1))
        };

    // check case of 2 nodes
    if possible_nodes.len() == 2 {
        return (Some(best_nodes.0.0), Some(best_nodes.1.0));
    }

    // if there's 3 or more nodes, check out the full list
    for (node_id, node_vigor) in possible_nodes.iter() {
        if node_vigor > &best_nodes.1.1 {

            if node_vigor > &best_nodes.0.1 {
                // reassign best nodes 2
                best_nodes.1 = best_nodes.0;
                best_nodes.0 = (*node_id, *node_vigor);
            } else {
                best_nodes.1 = (*node_id, *node_vigor);
            }
        }
    }

    // return the two best nodes
    (Some(best_nodes.0.0), Some(best_nodes.1.0))
}