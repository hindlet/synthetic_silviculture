#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use super::{
    super::{
        plants::plant::*,
        environment::{params::*, light_cells::*},
        maths::{vector_three::Vector3, matrix_three::Matrix3, lerp, bounding_sphere::BoundingSphere},
        graphics::branch_mesh_gen::MeshUpdateQueue,
    },
    branch::*,
    branch_node::*,
    branch_prototypes::*,
};



pub fn calculate_branch_light_exposure(
    mut branches_query: Query<(&mut BranchGrowthData, &BranchBounds), With<BranchTag>>,
    branch_connection_query: Query<&mut BranchConnectionData, With<BranchTag>>,
    plant_query: Query<(&PlantPlasticityParameters, &PlantData), With<PlantTag>>,

    mut light_cells: ResMut<LightCells>,
) {
    // update the light cells
    light_cells.set_all_zero();
    for (_growth_data, bounds) in branches_query.iter() {
        light_cells.add_volume_to_cell(bounds.bounds.centre, bounds.bounds.get_volume());
    }

    // update light exposure
    for (plasticity_params, plant_data) in plant_query.iter() {
        if plant_data.root_node.is_none() {continue;}
        let tolerance = plasticity_params.shadow_tolerance;

        for id in get_terminal_branches(&branch_connection_query, plant_data.root_node.unwrap()) {
            if let Ok((mut growth_data, bounds)) = branches_query.get_mut(id) {
                growth_data.light_exposure = lerp(tolerance, 1.0, light_cells.get_cell_light(bounds.bounds.centre / light_cells.size()))
            }
        }
    }
}

/// Calculates growth rates for all branches
/// 
/// Growth Rate = Sigmoid( (branch_vigor - plant_min_vigor)/(plant_max_vigor - plant_min_vigor) ) * plant_growth_rate
pub fn assign_growth_rates(
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





/// increases the physiological age of all the branches and their nodes by their growth rate
pub fn step_physiological_age(
    mut branch_query: Query<(&BranchData, &mut BranchGrowthData), With<BranchTag>>,
    node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    mut node_data_query: Query<&mut BranchNodeData, With<BranchNodeTag>>,

    age_step: Res<PhysicalAgeStep>,
) {
    for (data, mut growth_data) in branch_query.iter_mut() {
        growth_data.physiological_age += growth_data.growth_rate;
        if data.root_node.is_none() {continue;}
        for id in get_nodes_base_to_tip(&node_connections_query, data.root_node.unwrap()) {
            if let Ok(mut node_data) = node_data_query.get_mut(id) {
                node_data.phys_age += growth_data.growth_rate * age_step.step;
            }
        }
    }
}





/// updates the nodes on branches where required
/// 
/// The number of layers on the tree that stores the nodes will be lineraly interpolated between 1 and max 
/// using the age of the branch and the mature age of its prototype reference
pub fn update_branch_nodes(
    mut branch_data_query: Query<(&BranchData, &mut BranchGrowthData, &BranchPrototypeRef), With<BranchTag>>,

    branch_prototypes: Res<BranchPrototypes>,

    node_data_query: Query<&BranchNodeData, With<BranchNodeTag>>,
    mut node_connections_query: Query<&mut BranchNodeConnectionData, With<BranchNodeTag>>,

    mut commands: Commands
) {
    let prototype_data = branch_prototypes.get_age_layers_and_count();

    for (branch_data, mut branch_growth_data, prototype_ref) in branch_data_query.iter_mut() {
        // calculate how many layers the branch should currently have
        let target_layers = lerp(1.0,  prototype_data[prototype_ref.0].1 as f32, branch_growth_data.physiological_age / prototype_data[prototype_ref.0].0).round() as u32;

        // if the branch is missing layers, keep adding more
        while branch_growth_data.layers < target_layers {
            // println!("check");
            // get a list of nodes on the current layer and loop through them
            let current_layer = get_nodes_on_layer(&mut node_connections_query, branch_data.root_node.unwrap(), branch_growth_data.layers);
            // println!("{}", current_layer.len());
            
            for i in 0..current_layer.len() {
                // check how many children it should have, if none, skip
                let num_children = prototype_data[prototype_ref.0].2[branch_growth_data.layers as usize - 1][i];
                if num_children == 0 {continue;}
                // println!("{}", num_children);

                // get the parent node diameter
                let node_thickning_factor = {
                    if let Ok(node_data) = node_data_query.get(current_layer[i]) {
                        node_data.thickening_factor
                    }else {panic!()}
                };

                // create the new children and add them to the parent node, new nodes will take their parent's thickness
                if let Ok(mut node_connections) = node_connections_query.get_mut(current_layer[i]) {
                    for j in 0..num_children {
                        let id = commands.spawn(BranchNodeBundle {
                            connections: BranchNodeConnectionData{parent: Some(current_layer[i]), ..Default::default()},
                            data: BranchNodeData{thickening_factor: node_thickning_factor, phys_age: branch_growth_data.physiological_age, ..Default::default()},
                            ..Default::default() 
                        }).id();
                        node_connections.children.push(id);
                        // println!("{:?}", node_connections.children);
                    }
                }
            }
            branch_growth_data.layers += 1;
        }
        
    }
}






/// decides if and where to create new branches on a plant
pub fn determine_create_new_branches(
    plant_query: Query<(&PlantData, &PlantGrowthControlFactors), With<PlantTag>>,

    branch_data_query: Query<(&BranchData, &BranchGrowthData, &BranchPrototypeRef), With<BranchTag>>,
    branch_bounds_query: Query<&BranchBounds, With<BranchTag>>,
    mut branch_connections_query: Query<&mut BranchConnectionData, With<BranchTag>>,

    branch_prototypes_sampler: Res<BranchPrototypesSampler>,
    branch_prototypes: Res<BranchPrototypes>,

    mut node_data_query: Query<(&mut BranchNodeGrowthData, &BranchNodeData), With<BranchNodeTag>>,
    node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,

    gravity_res: Res<GravityResources>,

    mut commands: Commands
) {

    let prototypes = &branch_prototypes.prototypes;
    let tropism_dir = gravity_res.gravity_dir * (gravity_res.tropism_strength / gravity_res.tropism_strength.abs());

    for (plant_data, plant_growth_factors) in plant_query.iter() {
        if plant_data.root_node.is_none() {continue;}

        let branches_to_check = get_terminal_branches(&branch_connections_query, plant_data.root_node.unwrap());
        let v_min = plant_growth_factors.min_vigor;
        let v_max = plant_growth_factors.max_vigor;
        let apical = plant_growth_factors.apical_control;
        let plant_dist_control = plant_growth_factors.tropism_angle_weight;
        let plant_angle = plant_growth_factors.branching_angle;
        let max_branch_length = plant_growth_factors.max_branch_segment_length;


        let mut branch_bounds = Vec::new();

        // loop through every terminal branch
        for id in branches_to_check {
            if let Ok((branch_data, branch_growth_data, prototype_ref)) = branch_data_query.get(id) {
                
                if branch_data.root_node.is_none() {continue;}
                
                // branch must be older than mature for new branches to be added
                if branch_growth_data.physiological_age <= branch_prototypes.prototypes[prototype_ref.0].mature_age {continue;}

                // assign light exposure to terminal branch nodes
                let terminal_node_ids = get_terminal_nodes(&node_connections_query, branch_data.root_node.unwrap());
                let node_light_level = branch_growth_data.light_exposure / terminal_node_ids.len() as f32;
                for id in terminal_node_ids {
                    if let Ok(mut node_data) = node_data_query.get_mut(id) {
                        node_data.0.light_exposure = node_light_level;
                    }
                }

                let prototype_index = branch_prototypes_sampler.get_prototype_index(apical, branch_growth_data.growth_vigor * branch_prototypes_sampler.max_determinancy / v_max);
                if branch_bounds.len() == 0 {branch_bounds = get_branch_bounds_base_to_tip(&branch_bounds_query, &branch_connections_query, branch_data.root_node.unwrap())}

                // get the nodes on the branch that could generate new branches
                let mut possible_terminal_nodes = get_possible_new_branch_nodes(&mut node_data_query, &node_connections_query, branch_data.root_node.unwrap(), v_min, v_max, &prototypes[prototype_index], plant_angle, plant_dist_control, branch_data.normal, tropism_dir, max_branch_length, &branch_bounds);
                if possible_terminal_nodes.0.is_none() {continue;}


                if let Ok(mut connections) = branch_connections_query.get_mut(id) {
                    
                    if connections.children.0.is_none() {
                        let child_root_id = commands.spawn(BranchNodeBundle{data: BranchNodeData{thickening_factor: possible_terminal_nodes.0.unwrap().1, ..Default::default()}, ..Default::default()}).id();
                        let child_id = commands.spawn(BranchBundle{
                            data: BranchData {
                                parent_node: Some(possible_terminal_nodes.0.unwrap().0),
                                root_node: Some(child_root_id),
                                normal: branch_data.normal,
                                ..Default::default()
                            },
                            connections: BranchConnectionData{
                                parent: Some(id),
                                ..Default::default()
                            },
                            prototype: BranchPrototypeRef(prototype_index),
                            ..Default::default()
                        }).id();
                        possible_terminal_nodes.0 = None;
                        connections.children.0 = Some(child_id);
                    }
                    
                    if connections.children.1.is_none() {
                        if possible_terminal_nodes.0.is_none() && possible_terminal_nodes.1.is_none() {continue;}
                        let parent_node = {if possible_terminal_nodes.0.is_some() {possible_terminal_nodes.0} else {possible_terminal_nodes.1}};
                        let child_root_id = commands.spawn(BranchNodeBundle{data: BranchNodeData{thickening_factor: parent_node.unwrap().1, ..Default::default()}, ..Default::default()}).id();
                        let child_id = commands.spawn(BranchBundle{
                            data: BranchData {
                                parent_node: Some(parent_node.unwrap().0),
                                root_node: Some(child_root_id),
                                normal: branch_data.normal,
                                ..Default::default()
                            },
                            connections: BranchConnectionData{
                                parent: Some(id),
                                ..Default::default()
                            },
                            prototype: BranchPrototypeRef(prototype_index),
                            ..Default::default()
                        }).id();
                        connections.children.1 = Some(child_id);
                    }
                }


            }
        }
    }
}


/// Helper func to decide which nodes in a branch could generate a new branch
/// 
/// returns the two nodes on the branch that are the options for branch attachments
/// the data returned for each is an id, and the node's position and thickening factor
fn get_possible_new_branch_nodes(
    node_data_query: &mut Query<(&mut BranchNodeGrowthData, &BranchNodeData), With<BranchNodeTag>>,
    node_connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
    branch_min_vigor: f32,
    branch_max_vigor: f32,

    prototype_data: &BranchPrototypeData,
    plant_angle: f32,
    plant_distr_control: f32,
    parent_normal: Vector3,
    tropism_dir: Vector3,
    max_branch_length: f32,
    other_branch_bounds: &Vec<BoundingSphere>,


) -> (Option<(Entity, f32, Vector3)>, Option<(Entity, f32, Vector3)>) {
    

    // sum up light exposure at branching points
    for id in get_nodes_tip_to_base(&node_connections_query, root_node) {
        #[allow(unused_assignments)]
        let mut light_exposure = 0.0;
        if let Ok(node_data) = node_data_query.get(id) {
            light_exposure = node_data.0.light_exposure;
        } else {panic!("tried to get node: {:?}, and failed", id)}
        if let Ok(node_connections) = node_connections_query.get(id) {
            if node_connections.parent.is_none() {continue;}
            if let Ok(mut parent_data) = node_data_query.get_mut(node_connections.parent.unwrap()) {
                parent_data.0.light_exposure += light_exposure;
            }
        }
    }

    if let Ok(mut root_node_data) = node_data_query.get_mut(root_node) {
        root_node_data.0.growth_vigor = root_node_data.0.light_exposure;
    }

    // distribute vigor up the branch, there is no apical control so it's slightly easier
    for id in get_nodes_base_to_tip(&node_connections_query, root_node) {
        #[allow(unused_assignments)]
        let mut parent_vigor = 0.0;        

        if let Ok(parent_data) = node_data_query.get(id) {
            parent_vigor = parent_data.0.growth_vigor;
        } else {panic!("Oh god oh no the end is nigh")}

        if let Ok(parent_connections) = node_connections_query.get(id) {

            if parent_connections.children.len() == 0 {continue;}
            if parent_connections.children.len() == 1 {
                if let Ok(mut only_child) = node_data_query.get_mut(parent_connections.children[0]) {
                    only_child.0.growth_vigor = parent_vigor;
                    continue;
                }
            }

            // calculate the sum of all the child light exposures
            let child_light_sum = {
                let mut light_sum = 0.0;

                for id in parent_connections.children.iter() {
                    if let Ok(child_data) = node_data_query.get(*id) {
                        light_sum += child_data.0.light_exposure;
                    }
                }
                light_sum
            };

            // distribute parent vigor to children
            for id in parent_connections.children.iter() {
                if let Ok(mut child_data) = node_data_query.get_mut(*id) {
                    child_data.0.growth_vigor = parent_vigor * (child_data.0.light_exposure/child_light_sum);
                }
            }
        }
    }

    
    // get the possible nodes
    let mut possible_nodes = Vec::new();

    for id in get_terminal_nodes(&node_connections_query, root_node) {
        if let Ok(node_data) = node_data_query.get(id) {
            if node_data.0.growth_vigor <= branch_min_vigor {continue;}

            let (normal, weight) = get_new_normal(plant_angle, plant_distr_control, parent_normal, tropism_dir, prototype_data, max_branch_length, node_data.1.position + node_data.1.tropism_offset, other_branch_bounds);

            possible_nodes.push(((id, node_data.1.thickening_factor, normal), weight));
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





/// assigns the node thicknesses using: 
/// node_diameter = if (has_children) {root(sum(child_diameter_sqrd))} else {node_thickening_factor}
pub fn assign_thicknesses(
    plant_query: Query<&PlantData, With<PlantTag>>,
    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
    branch_query: Query<&BranchData, With<BranchTag>>,
    node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    mut node_data_query: Query<&mut BranchNodeData, With<BranchNodeTag>>,
) {

    for plant in plant_query.iter() {

        if plant.root_node.is_none() {continue;}

        for branch_id in get_branches_tip_to_base(&branch_connections_query, plant.root_node.unwrap()){

            if let Ok(branch) = branch_query.get(branch_id) {

                if branch.root_node.is_none() {continue;}

                // distribute thick down the plant
                for node_id in get_nodes_tip_to_base(&node_connections_query, branch.root_node.unwrap()) {

                    let mut child_thick_sqr_sum = 0.0;
                    if let Ok(connection_data) = node_connections_query.get(node_id) {

                        for child_id in connection_data.children.iter() {
                            if let Ok(child_data) = node_data_query.get(*child_id) {
                                child_thick_sqr_sum += child_data.thickness * child_data.thickness;
                            }
                        }
                    }

                    if let Ok(mut data) = node_data_query.get_mut(node_id) {
                        if child_thick_sqr_sum == 0.0 {
                            if data.thickness > 0.0 {continue;}
                            data.thickness = data.thickening_factor;
                        } else {
                            data.thickness = child_thick_sqr_sum.sqrt();
                        }

                    }
                }

                // send base thick to the parent node 
                if branch.parent_node.is_none() {continue;}


                let root_thick = {
                    if let Ok(root_data) = node_data_query.get(branch.root_node.unwrap()) {
                        root_data.thickness
                    } else {panic!("Failed to get root node")}
                };

                if let Ok(mut parent_data) = node_data_query.get_mut(branch.parent_node.unwrap()) {
                    parent_data.thickness = root_thick;
                }
            }
        }
    }
}





/// calculates the positions of all the branch nodes using:
/// segment_length = min(max_segment_length, segment_length_scale * parent_node_physiological_age)
pub fn calculate_segment_lengths_and_tropism(
    plant_query: Query<(&PlantData, &PlantGrowthControlFactors), With<PlantTag>>,

    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
    mut branch_data_query: Query<(&mut BranchData, &BranchGrowthData, &BranchPrototypeRef), With<BranchTag>>,

    branch_prototypes: Res<BranchPrototypes>,

    mut branch_node_query: Query<&mut BranchNodeData, With<BranchNodeTag>>,
    branch_node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,

    gravity_res: Res<GravityResources>,
    mut queue: Query<&mut MeshUpdateQueue>
) {

    let directions = branch_prototypes.get_directions();
    let ages = branch_prototypes.get_ages();
    let grav = gravity_res.gravity_dir * gravity_res.tropism_strength;
    let mut queue = queue.single_mut();


    for (plant_data, plant_growth_factors) in plant_query.iter() {

        if plant_data.root_node.is_none() {continue;}

        let max_length = plant_growth_factors.max_branch_segment_length;
        let scale = plant_growth_factors.branch_segment_length_scaling_coef;
        let plant_tropism = plant_growth_factors.tropism_time_control;

        for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {

            let (parent_offset, parent_rotation_matrix) = {
                let parent_id = get_branch_parent_id(id, &branch_connections_query);
                
                if parent_id.is_none() {
                    (Vector3::ZERO(), Matrix3::identity())
                }
                else if let Ok(branch_parent_data) = branch_data_query.get(parent_id.unwrap()) {
                    
                    let rotation_matrix = {
                        let rotation_axis = branch_parent_data.0.normal.cross(Vector3::Y());
                        let rotation_angle = branch_parent_data.0.normal.angle_to(Vector3::Y());
                        Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
                    };
                    (branch_parent_data.0.root_position, rotation_matrix)

                } else {
                    (Vector3::ZERO(), Matrix3::identity())
                }
            };


            if let Ok((mut branch_data, branch_growth_data, prototype_ref)) = branch_data_query.get_mut(id) {

                if branch_data.root_node.is_none() {continue;}
                if branch_data.full_grown {continue;}
                if branch_growth_data.physiological_age > ages[prototype_ref.0] {
                    branch_data.full_grown = true;
                }
                let branch_age = branch_growth_data.physiological_age;


                if branch_data.parent_node.is_some() {
                    // update the root node position based on the position of the parent node (they're the same)
                    let root_position = {
                        if let Ok(parent_node) = branch_node_query.get(branch_data.parent_node.unwrap()) {
                            parent_node.position + parent_node.tropism_offset
                        } else {panic!("failed to get branch parent node")}
                    };
                    branch_data.root_position = root_position.transform(parent_rotation_matrix) + parent_offset;
                }

                // update all node positions
                let node_pairs = get_nodes_and_connections_base_to_tip(&branch_node_connections_query, branch_data.root_node.unwrap());

                if node_pairs.len() == 0 {continue;}

                for i in 0..node_pairs.len() {
                    if let Ok(mut node_pair) = branch_node_query.get_many_mut(node_pairs[i]) {
                        
                        let segment_age = (branch_age - node_pair[1].phys_age).max(0.0);


                        let length = max_length.min(scale * segment_age);

                        let new_offset = directions[prototype_ref.0][i] * length;
                        let tropism_offset = grav * plant_tropism * length / max_length;


                        node_pair[1].tropism_offset = tropism_offset;
                        node_pair[1].position = node_pair[0].position + new_offset;

                        if node_pair[1].position == Vector3::ZERO() && node_pairs[i][0] != branch_data.root_node.unwrap() {
                            panic!("Node positioned wrong, nodes: {:?}, {:?} \ndata: \nage: {:?} \nlength: {:?} \noffset: {:?} \nparent_pos: {:?}", node_pair[0], node_pair[1], segment_age, length, new_offset, node_pair[0].position);
                        }
                    }else {panic!("Could not get node pair, tried to get nodes: {:?}, {:?}", node_pairs[i][0], node_pairs[i][1])}
                }

                queue.0.push_back(id);
            }
        }
    }
}







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
                node_positions.push(node_data.position.clone().transform(branch_rotation_matrix) + node_data.tropism_offset);
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