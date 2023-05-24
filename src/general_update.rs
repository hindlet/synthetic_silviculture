use bevy_ecs::prelude::*;
use itertools::*;
use super::{
    plants::plant::{PlantData, PlantBounds, PlantTag},
    branches::{
        branch::{BranchGrowthData, BranchTag, BranchConnectionData, BranchData, BranchBundle, BranchBounds, get_branches_base_to_tip},
        branch_node::{BranchNodeGrowthData, BranchNodeConnectionData, BranchNodeTag, get_terminal_nodes, get_nodes_tip_to_base, get_nodes_base_to_tip, get_nodes_and_connections_base_to_tip, BranchNodeData, get_nodes_on_layer, BranchNodeBundle},
        branch_prototypes::{BranchPrototypes, BranchPrototypeRef, BranchPrototypesSampler},
    },
    maths::{vector_three::Vector3, matrix_three::Matrix3, bounding_sphere::BoundingSphere, bounding_box::BoundingBox},
};


// updates branch bounds
pub fn update_branch_bounds(
    node_data: Query<&BranchNodeData, With<BranchNodeTag>>,
    nodes_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    mut branches_query: Query<(&mut BranchBounds, &BranchData), With<BranchTag>>
) {
    for (mut bounds, data) in &mut branches_query {
        if data.root_node.is_none() {continue;}

        let branch_rotation_matrix = {
            let mut rotation_axis = data.normal.cross(Vector3::Y());
            rotation_axis.normalise();
            let rotation_angle = data.normal.angle_to(Vector3::Y());
            Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
        };

        let mut node_positions: Vec<Vector3> = Vec::new();

        for id in get_nodes_base_to_tip(&nodes_connections_query, data.root_node.unwrap()) {
            if let Ok(node_data) = node_data.get(id) {
                node_positions.push(node_data.position.clone().transform(branch_rotation_matrix));
            }
        }

        let mut new_bounds = 
            if node_positions.len() == 1 {
                BoundingSphere::new(node_positions[0], 0.01)
            }
            else {
                BoundingSphere::from_points(node_positions)
            };
        
        new_bounds.centre += data.root_position;

        bounds.bounds = new_bounds;
    }
}

/// calculates branch intersection volumes
// pub fn calculate_branch_intersection_volumes(
//     mut branch_query: Query<(&mut BranchData, &BranchBounds, Entity), With<BranchTag>>,
// ) {
//     let mut intersection_lists: Vec<(Entity, BoundingSphere, Vec<Entity>)> = Vec::new();
//     for (mut data, bounds, id) in branch_query.iter_mut() {
//         data.intersections_volume = 0.0;
//         let mut intersections = Vec::new();
//         for id_other in data.intersection_list.iter() {
//             intersections.push(*id_other);
//         }
//         intersection_lists.push((id, bounds.bounds.clone(), intersections));
//     }

//     for branch_one in intersection_lists {
//         let mut volume = 0.0;
//         for id in branch_one.2.iter() {
//             if let Ok(mut branch_two) = branch_query.get_mut(*id) {
//                 let intersection = branch_one.1.get_intersection_volume(&branch_two.1.bounds);
//                 branch_two.0.intersections_volume += intersection;
//                 volume += intersection;
//             }
//         }
//         if let Ok(mut branch) = branch_query.get_mut(branch_one.0) {
//             branch.0.intersections_volume += volume;
//         }
//     }

// }


// must be called after updating all the branch bounds
pub fn update_plant_bounds(
    branch_bounds_query: Query<&BranchBounds, With<BranchTag>>,
    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
    mut plants_query: Query<(&mut PlantBounds, &PlantData), With<PlantTag>>
) {
    for (mut bounds, plant_data) in &mut plants_query {
        if plant_data.root_node.is_none() {continue;}

        let mut branch_bounds: Vec<BoundingSphere> = Vec::new();

        
        for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {
            if let Ok(bounds) = branch_bounds_query.get(id) {
                branch_bounds.push(bounds.bounds.clone());
            }
        }

        let new_bounds = BoundingBox::from_spheres(branch_bounds);
        bounds.bounds = new_bounds;
    }
}

// this will calculate all the plant intersections, it will not contain any repeated intersect
pub fn update_plant_intersections(
    mut plants_query: Query<(&PlantBounds, &mut PlantData, Entity), With<PlantTag>>,
) {
    // reset all plant intersection lists
    for (_bounds, mut data, _id) in &mut plants_query {
        data.intersection_list = Vec::new();
    }
    // check all plant intersection options
    let mut combinations = plants_query.iter_combinations_mut();
    while let Some([mut plant_one, plant_two]) = combinations.fetch_next() {
        if plant_one.0.bounds.is_intersecting_box(plant_two.0.bounds) {
            plant_one.1.intersection_list.push(plant_two.2);
        }
    }
}


// /// this relies on the fact that our plant intersections will not contain any repeats,
// /// if they did the branches would end up with double the intersection volumes they are meant to
// pub fn update_branch_intersections(
//     plants_query: Query<&PlantData, With<PlantTag>>,
//     mut branch_query: Query<(&BranchBounds, &mut BranchData), With<BranchTag>>,
//     branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
// ) {
//     // loop through each plant
//     for plant_data in plants_query.iter() {
//         if plant_data.root_node.is_none() {continue;}
        
//         // loop through intersections
//         for other_plant_id in plant_data.intersection_list.iter() {

//             // get a list of the bounds of the other plants branches
//             let mut other_plant_branch_bounds: Vec<(BoundingSphere, Entity)> = vec![];

//             // loop through all the branches we could intersect with and add them to a list
//             if let Ok(other_plant) = plants_query.get(*other_plant_id) {
//                 if other_plant.root_node.is_none() {continue;}
//                 for id in get_branches_base_to_tip(&branch_connections_query, other_plant.root_node.unwrap()) {
//                     if let Ok(branch) = &branch_query.get(id) {
//                         other_plant_branch_bounds.push((branch.0.bounds.clone(), id));
//                     }
//                 }
//             }

//             // loop through each of our branches
//             for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {
//                 if let Ok(mut branch) = branch_query.get_mut(id) {
//                     // reset the branches intersections list and volume
//                     branch.1.intersection_list = Vec::new();
//                     branch.1.intersections_volume = 0.0;
//                     // check if the branches intersect, if so, add the second branch id to the first's list
//                     for other_bounds in other_plant_branch_bounds.iter() {
//                         if branch.0.bounds.is_intersecting_sphere(&other_bounds.0) {
//                             branch.1.intersection_list.push(other_bounds.1);
//                         }
//                     }
//                 }
//             } 

//             // check through our own branches for collissions
//             // I don't like this code but i had to fight the borrow checker
//             for combination in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()).iter().combinations(2) {
//                 let other_data: BoundingSphere;
//                 if let Ok(branch_two) = branch_query.get(*combination[1]){
//                     other_data = branch_two.0.bounds.clone();
//                 } else {panic!("Fuck balls shit fuck balls")};
//                 if let Ok(mut branch_one) = branch_query.get_mut(*combination[0]){
//                     if branch_one.0.bounds.is_intersecting_sphere(&other_data) {
//                         branch_one.1.intersection_list.push(*combination[1]);
//                     }
//                 };
                
//             }
//         }

//     }
// }