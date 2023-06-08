#![allow(dead_code, unused_variables, unused_imports)]
use std::{time::{Duration, Instant}, rc::Rc, cell::RefCell, borrow::{BorrowMut, Borrow}};
use super::{
    super::{
        plants::plant::*,
        environment::{light_cells::*, *},
        maths::{vector_three::Vector3, matrix_three::Matrix3, lerp, bounding_sphere::BoundingSphere, quicksort},
    },
    branch::*,
    branch_node::*,
    branch_prototypes::*,
    branch_sorting::*,
    node_sorting::*,
};



pub fn calculate_branch_light_exposure(
    plants: &Vec<Plant>,

    light_cells: &mut LightCells,
) {
    // update the light cells
    light_cells.set_all_zero();
    for branch_cell in get_all_branches(plants) {
        let branch = branch_cell.as_ref().borrow();
        light_cells.add_volume_to_cell(branch.bounds.centre / light_cells.size(), branch.bounds.get_volume());
    }

    // update light exposure
    for plant in plants.iter() {
        let shadow_tolerance = plant.plasticity.shadow_tolerance;

        for branch_cell in branches_base_to_tip(&plant.root) {
            let mut branch = branch_cell.as_ref().borrow_mut();
            branch.growth_data.light_exposure = lerp(shadow_tolerance, 1.0, light_cells.get_cell_light(branch.bounds.centre / light_cells.size()));
        }

    }

}

/// Calculates growth rates for all branches
/// 
/// Growth Rate = Sigmoid( (branch_vigor - plant_min_vigor)/(plant_max_vigor - plant_min_vigor) ) * plant_growth_rate
pub fn assign_growth_rates(
    plants: &Vec<Plant>
) {
    for plant in plants.iter() {
        let v_min = plant.growth_factors.min_vigor;
        let v_max = plant.growth_factors.max_vigor;
        let plant_growth = plant.growth_factors.growth_rate;

        for branch_cell in branches_base_to_tip(&plant.root) {
            let mut branch = branch_cell.as_ref().borrow_mut();
            branch.growth_data.growth_rate = sigmoid((branch.growth_data.growth_vigor - v_min) / (v_max - v_min)) * plant_growth;
        }

    }
}

fn sigmoid(x: f32) -> f32 {
    3.0 * x.powi(2) - 2.0 * x.powi(3)
}





/// increases the physiological age of all the branches and their nodes by their growth rate
pub fn step_physiological_age(
    plants: &Vec<Plant>,
    age_step: f32,
) {
    for branch_cell in get_all_branches(plants) {
        let mut branch = branch_cell.as_ref().borrow_mut();
        branch.growth_data.physiological_age += branch.growth_data.growth_rate * age_step;
        for node_cell in nodes_base_to_tip(&branch.root) {
            node_cell.as_ref().borrow_mut().data.phys_age += age_step;
        }
    }
}





/// updates the nodes on branches where required
/// 
/// The number of layers on the tree that stores the nodes will be lineraly interpolated between 1 and max 
/// using the age of the branch and the mature age of its prototype reference
pub fn update_branch_nodes(
    plants: &Vec<Plant>,
    branch_prototypes: &BranchPrototypes,
) {
    let prototype_data = branch_prototypes.get_age_layers_and_count();

    for branch_cell in get_all_branches(plants) {
        let mut branch = branch_cell.as_ref().borrow_mut();
        // calculate how many layers the branch should currently have
        let target_layers = lerp(1.0, prototype_data[branch.prototype_id].1 as f32, branch.growth_data.physiological_age / prototype_data[branch.prototype_id].0).floor() as u32;

        // if the branch is missing layers, add one more laye
        if branch.growth_data.layers < target_layers {

            let working_layer_nodes = get_nodes_on_layer(&branch.root, branch.growth_data.layers);
            let layer_child_counts = prototype_data[branch.prototype_id].2[branch.growth_data.layers as usize - 1].clone();

            for i in 0..working_layer_nodes.len() {
                // check how many children it should have, if none, skip
                let num_children = layer_child_counts[i];
                if num_children == 0 {continue;}

                let mut parent_node = working_layer_nodes[i].as_ref().borrow_mut();
                let parent_thickening_factor = parent_node.data.thickening_factor;

                for j in 0..num_children {
                    parent_node.children.push(
                        Rc::new(RefCell::new(BranchNode {
                            data: BranchNodeData {thickening_factor: parent_thickening_factor, phys_age: branch.growth_data.physiological_age, ..Default::default()},
                            ..Default::default()
                        }))
                    )
                }
            }

            branch.growth_data.layers += 1;
        }

    }
}






/// decides if and where to create new branches on a plant
pub fn determine_create_new_branches(
    plants: &Vec<Plant>,

    branch_prototypes_sampler: &BranchPrototypesSampler,
    branch_prototypes: &BranchPrototypes,

    gravity_res: &GravitySettings,
) {

    let prototypes = &branch_prototypes.prototypes;
    let tropism_dir = gravity_res.gravity_dir * (gravity_res.tropism_strength / gravity_res.tropism_strength.abs());

    for plant in plants.iter() {

        let v_min = plant.growth_factors.min_vigor;
        let v_max = plant.growth_factors.max_vigor;
        let apical = plant.growth_factors.apical_control;
        let plant_distr_control = plant.growth_factors.tropism_angle_weight;
        let plant_angle = plant.growth_factors.branching_angle;
        let max_branch_length = plant.growth_factors.max_branch_segment_length;

        let branch_bounds = get_branch_bounds(&plant.root);

        for (branch_cell, index) in branches_terminal_with_index(&plant.root) {
            let mut branch = branch_cell.as_ref().borrow_mut();

            // only branches where age > mature_age can have new branches attached
            if branch.growth_data.physiological_age <= prototypes[branch.prototype_id].mature_age {continue;}

            // check how many more children the branch needs, if none continue
            let num_needed_children: u32 = if branch.children.1.is_some() {0} else if branch.children.0.is_some() {1} else {2};
            if num_needed_children == 0 {continue;}

            let terminal_node_cells = terminal_nodes(&branch.root);
            let terminal_node_light = branch.growth_data.light_exposure / terminal_node_cells.len() as f32;
            for cell in terminal_node_cells.iter() {
                cell.as_ref().borrow_mut().growth_data.light_exposure = terminal_node_light;
            }

            // get the new prototype index: Determinancy = parent_vigor * max_det / v_max
            let new_prototype_index = branch_prototypes_sampler.get_prototype_index(apical, branch.growth_data.growth_vigor * branch_prototypes_sampler.max_determinancy / v_max);

            let layers = get_node_layers(&branch.root);

            dist_node_vigor(layers);
            let mut possible_nodes = get_best_terminal_nodes(terminal_node_cells, branch.data.root_position, v_min, &prototypes[new_prototype_index], plant_angle, plant_distr_control, branch.data.normal, tropism_dir, max_branch_length, &branch_bounds);

            if possible_nodes.len() == 0 {continue;}

            // children.0
            if num_needed_children == 2 {
                let data = possible_nodes.remove(0);
                branch.children.0 = Some(Rc::new(RefCell::new(Branch::new(data.3, data.1, data.2, new_prototype_index, data.0, index))));
            }
            
            if possible_nodes.len() == 0 {continue;}

            // children.1
            let data = possible_nodes.remove(0);
            branch.children.1 = Some(Rc::new(RefCell::new(Branch::new(data.3, data.1, data.2, new_prototype_index, data.0, index))));
        }

    }

}

/// takes in a list of node layers from base to tip and distributes light exposure down and then growth vigor up
fn dist_node_vigor(
    mut node_cells: Vec<Vec<Rc<RefCell<BranchNode>>>>,
) {

    // distribute light down
    node_cells.reverse();
    for i in 0..node_cells.len() - 1 {

        for cell in node_cells[i + 1].iter() {
            cell.as_ref().borrow_mut().growth_data.light_exposure = 0.0;
        }

        for cell in node_cells[i].iter() {
            let node = cell.as_ref().borrow();
            node_cells[i + 1][node.parent].as_ref().borrow_mut().growth_data.light_exposure += node.growth_data.light_exposure;
        }
    }

    // reverse and convert light to vigor
    node_cells.reverse();
    {
        let mut node = node_cells[0][0].as_ref().borrow_mut();
        node.growth_data.growth_vigor = node.growth_data.light_exposure;
    }

    // distribute vigor up
    for i in 0..node_cells.len() - 1 {

        for cell in node_cells[i].iter() {
            let node = cell.as_ref().borrow_mut();

            let mut child_light_sum = 0.0;
            for child_cell in node.children.iter() {
                child_light_sum += child_cell.as_ref().borrow().growth_data.light_exposure;
            }
            for child_cell in node.children.iter() {
                let mut child = child_cell.as_ref().borrow_mut();
                child.growth_data.growth_vigor = node.growth_data.growth_vigor * child.growth_data.light_exposure / child_light_sum;
            }

        }
    }

}

/// returns a sorted list of the terminal nodes based on the "distribution" fn below
/// the usize given is the node's index in the branch's terminal nodes list
fn get_best_terminal_nodes(
    terminal_nodes: Vec<Rc<RefCell<BranchNode>>>,
    
    branch_root_pos: Vector3,
    min_vigor: f32,

    prototype_data: &BranchPrototypeData,
    plant_angle: f32,
    plant_distr_control: f32,
    parent_normal: Vector3,
    tropism_dir: Vector3,
    max_branch_length: f32,
    other_branch_bounds: &Vec<BoundingSphere>,

) -> Vec<(usize, f32, Vector3, Vector3)>{

    let mut out_data = Vec::new();

    for i in 0..terminal_nodes.len() {
        let node = terminal_nodes[i].as_ref().borrow();
        if node.growth_data.growth_vigor <= min_vigor {continue;}
        let pos = node.data.relative_position + branch_root_pos;

        let (normal, weight) = get_new_normal(plant_angle, plant_distr_control, parent_normal, tropism_dir, prototype_data, max_branch_length, pos, other_branch_bounds);
        out_data.push((weight, (i, node.data.thickening_factor, normal, pos)));
    }

    let sorted = quicksort(out_data);
    let mut out = Vec::new();
    for val in sorted {
        out.push(val.1);
    }

    out
}


/// helper function to calculate the normal of a new branch module
/// 
/// The normal is based on 1 of 4 options, whichever has the smallest value from the distribution fn
fn get_new_normal(
    plant_angle: f32,
    plant_dist_control: f32,
    parent_normal: Vector3,
    tropism_dir: Vector3,
    prototype_data: &BranchPrototypeData,
    max_branch_length: f32,
    root_pos: Vector3,
    other_branch_bounds: &Vec<BoundingSphere>,

) -> (Vector3, f32) {
    let parent_angles = Vector3::direction_to_euler_angles(parent_normal);
    let angles_set = vec![Vector3::X() * plant_angle, Vector3::X() * -plant_angle, Vector3::Z() * plant_angle, Vector3::Z() * -plant_angle];
    let bounds_set = prototype_data.get_possible_bounds(max_branch_length, parent_angles, &angles_set, root_pos);
    let mut best: (Vector3, f32) = (Vector3::ZERO(), -100000.0);

    for i in 0..bounds_set.len() {
        let normal = Vector3::euler_angles_to_direction(parent_angles + angles_set[i]);
        let likelyhood = distribution(bounds_set[i], other_branch_bounds, tropism_dir.angle_to(normal), plant_angle, plant_dist_control);
        if likelyhood > best.1 {best = (normal, likelyhood)}
    }

    best
}


fn distribution(
    bounds: BoundingSphere,
    other_bounds: &Vec<BoundingSphere>,
    tropism_angle: f32,
    other_angle: f32,
    weight_one: f32
) -> f32{
    weight_one * tropism(tropism_angle, other_angle) + (1.0-weight_one) * possible_collisions_volume(bounds, other_bounds)
}

fn tropism(
    tropism_angle: f32,
    other_angle: f32
) -> f32{
    (tropism_angle.cos() - other_angle.cos()).abs()
}


/// only checks for collisions with branches inside of the plant
fn possible_collisions_volume(
    bounds: BoundingSphere,
    other_bounds: &Vec<BoundingSphere>,
) -> f32{
    let mut total_volume = 0.0;
    for to_check in other_bounds.iter() {
        if (bounds.radius + to_check.radius) * (bounds.radius + to_check.radius) < (bounds.centre - to_check.centre).sqr_magnitude() {continue;}
        total_volume += bounds.get_intersection_volume(to_check)
    }
    total_volume
}





/// assigns the node radii using: 
/// node_diameter = if (has_children) {root(sum(child_radius_squared))} else {node_thickening_factor}
pub fn assign_thicknesses(
    plants: &Vec<Plant>
) {

    let mut branch_list = get_all_branches(plants);
    branch_list.reverse();

    for branch_cell in branch_list {
        let branch = branch_cell.as_ref().borrow();

        let node_layers = get_node_layers(&branch.root);

        // assign the radii to node indices that do not exist
        let mut child_one_radius = (node_layers[node_layers.len()].len(), 0.0);
        let mut child_two_radius = (node_layers[node_layers.len()].len(), 0.0);

        if let Some(child_one_cell) = &branch.children.0 {
            let child = child_one_cell.as_ref().borrow();
            child_one_radius = (child.parent_node_index, child.root.as_ref().borrow().data.radius);
        }

        if let Some(child_two_cell) = &branch.children.1 {
            let child = child_two_cell.as_ref().borrow();
            child_two_radius = (child.parent_node_index, child.root.as_ref().borrow().data.radius);
        }

        
        for i in (0..node_layers.len()).rev() {
            for j in 0..node_layers[i].len() {

                let mut node = node_layers[i][j].as_ref().borrow_mut();

                if i == node_layers.len() - 1 {
                    if j == child_one_radius.0 {
                        node.data.radius = child_one_radius.1;
                        continue;
                    }
                    else if j == child_two_radius.0 {
                        node.data.radius = child_two_radius.1;
                        continue;
                    }
                }

                if node.children.len() == 0 && node.data.radius == 0.0 {node.data.radius = node.data.thickening_factor; continue;}

                let mut squared_child_sum = 0.0;
                for child in node.children.iter() {
                    squared_child_sum += child.as_ref().borrow().data.radius;
                }
    
                node.data.radius = squared_child_sum.sqrt();
            }
        }
    }
}





/// calculates the positions of all the branch nodes using:
/// segment_length = min(max_segment_length, segment_length_scale * parent_node_physiological_age)
pub fn calculate_segment_lengths_and_tropism(
    plants: &Vec<Plant>,

    branch_prototypes: &BranchPrototypes,
    gravity_res: &GravitySettings,
) {

    let directions = branch_prototypes.get_directions();
    let ages = branch_prototypes.get_ages();
    let grav = gravity_res.gravity_dir * gravity_res.tropism_strength;

    for plant in plants.iter() {
        let max_length = plant.growth_factors.max_branch_segment_length;
        let branch_scale = plant.growth_factors.branch_segment_length_scaling_coef;
        let plant_tropism = plant.growth_factors.tropism_control;


        for branch_cell in branches_base_to_tip(&plant.root) {

            let mut branch = branch_cell.as_ref().borrow_mut();

            if branch.data.finalised_mesh {continue;}
            if branch.growth_data.physiological_age > ages[branch.prototype_id] {
                branch.data.finalised_mesh = true;
            }

            let rotation_mat = Matrix3::from_angle_and_axis(branch.data.normal.angle_to(Vector3::Y()), branch.data.normal.cross(Vector3::Y()));


            let layers = get_node_layers(&branch.root);

            // if there are no children, the index will be outside the range so will not be checked
            let child_one_index = {
                if let Some(child_branch) = &branch.children.0 {
                    branch.borrow().parent_node_index
                }
                else {layers[layers.len() - 1].len()}
            };

            let child_two_index = {
                if let Some(child_branch) = &branch.children.1 {
                    branch.borrow().parent_node_index
                }
                else {layers[layers.len() - 1].len()}
            };

            let branch_age = branch.growth_data.physiological_age;

            // update branch node positions
            for i in 0..layers.len() - 1 {

                for node_cell in layers[i].iter() {
                    let node = node_cell.as_ref().borrow();
                    let parent_pos = node.data.relative_position;

                    for child_cell in node.children.iter() {
                        let mut child = child_cell.as_ref().borrow_mut();

                        let segment_age = (branch_age - child.data.phys_age).max(0.0);
                        let segment_length = (branch_scale * segment_age).min(max_length);
        
                        let offset = directions[branch.prototype_id][i] * segment_length;
                        let tropism_offset = grav * plant_tropism * segment_length;
        
                        let new_position = parent_pos + offset;
        
                        child.data.relative_position = new_position.transform(rotation_mat) + tropism_offset;
                    }
                }
            }

            // send positions to child
            for i in 0..layers[layers.len() - 1].len() {
                if i == child_one_index {
                    branch.children.0.as_ref().unwrap().as_ref().borrow_mut().data.root_position = layers[layers.len() - 1][i].as_ref().borrow().data.relative_position + branch.data.root_position;
                }
                if i == child_two_index {
                    branch.children.1.as_ref().unwrap().as_ref().borrow_mut().data.root_position = layers[layers.len() - 1][i].as_ref().borrow().data.relative_position + branch.data.root_position;
                }
            }

            branch.needs_mesh_update = Some(Instant::now());

        }

    }
}





// updates branch bounds
pub fn update_branch_bounds(
    plants: &Vec<Plant>
) {

    for branch_cell in get_all_branches(plants) {

        let mut branch = branch_cell.as_ref().borrow_mut();
        let root_pos = branch.data.root_position;

        let mut node_positions = Vec::new();
        for node in nodes_base_to_tip(&branch.root) {
            node_positions.push(node.as_ref().borrow().data.relative_position);
        }

        if node_positions.len() <= 1 {
            branch.bounds = BoundingSphere::new(branch.data.root_position, 0.01);
        }
        else {
            branch.bounds = BoundingSphere::from_points(node_positions)
        }
        branch.bounds.centre += root_pos;

    }
}

// /// calculates branch intersection volumes
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