#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::{
    plant::*,
    branch::*,
    branch_node::*,
    branch_prototypes::*,
    environment::{GravityResources, PhysicalAgeStep},
    maths::{vector_three::Vector3, matrix_three::Matrix3, lerp},
    graphics::branch_mesh_gen::MeshUpdateQueue,
    light_cells::LightCells,
};



pub fn calculate_branch_light_exposure(
    mut branches_query: Query<(&mut BranchGrowthData, &BranchBounds), With<BranchTag>>,
    branch_connection_query: Query<&BranchConnectionData, With<BranchTag>>,
    plant_query: Query<(&PlantPlasticityParameters, &PlantData), With<PlantTag>>,

    mut light_cells: ResMut<LightCells>,
) {
    // update the light cells
    light_cells.set_all_zero();
    for (_growth_data, bounds) in branches_query.iter() {
        light_cells.add_volume_to_cell(bounds.bounds.centre, bounds.bounds.get_volume());
    }

    // update light exposure
    for (plant_params, plant_data) in plant_query.iter() {
        if plant_data.root_node.is_none() {continue;}

        for id in get_branches_base_to_tip(&branch_connection_query, plant_data.root_node.unwrap()) {
            if let Ok((mut growth_data, bounds)) = branches_query.get_mut(id) {
                growth_data.light_exposure = lerp(plant_params.shadow_tolerance, 1.0, light_cells.get_cell_light(bounds.bounds.centre))
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


/// linearly interpolates between 1 and range, using the position of current in 0->max
fn old_lerp(
    max: f32,
    current: f32,
    range: u32,
) -> u32 {
    let position = current / max;
    let out = position * (range - 1) as f32;
    (1 + out.round() as u32).min(range)
}






/// decides if and where to create new branches on a plant
pub fn determine_create_new_branches(
    plant_query: Query<(&PlantData, &PlantGrowthControlFactors), With<PlantTag>>,

    branch_data_query: Query<(&BranchData, &BranchGrowthData, &BranchPrototypeRef), With<BranchTag>>,
    mut branch_connections_query: Query<&mut BranchConnectionData, With<BranchTag>>,

    branch_prototypes_sampler: Res<BranchPrototypesSampler>,
    branch_prototypes: Res<BranchPrototypes>,

    mut node_data_query: Query<(&mut BranchNodeGrowthData, &BranchNodeData), With<BranchNodeTag>>,
    node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,

    mut commands: Commands
) {
    for (plant_data, plant_growth_factors) in plant_query.iter() {
        if plant_data.root_node.is_none() {continue;}

        let branches_to_check = get_terminal_branches(&branch_connections_query, plant_data.root_node.unwrap());
        let v_min = plant_growth_factors.min_vigor;
        let v_max = plant_growth_factors.max_vigor;
        let apical = plant_growth_factors.apical_control;

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

                // get the nodes on the branch that could generate new branches
                let mut possible_terminal_nodes = get_possible_new_branch_nodes(&mut node_data_query, &node_connections_query, branch_data.root_node.unwrap(), v_min);
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
                            prototype: BranchPrototypeRef(branch_prototypes_sampler.get_prototype_index(apical, branch_growth_data.growth_vigor * branch_prototypes_sampler.max_determinancy / v_max)),
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
                            prototype: BranchPrototypeRef(branch_prototypes_sampler.get_prototype_index(apical, branch_growth_data.growth_vigor * branch_prototypes_sampler.max_determinancy / v_max)),
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
) -> (Option<(Entity, f32)>, Option<(Entity, f32)>) {
    

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

            possible_nodes.push(((id, node_data.1.thickening_factor), node_data.0.growth_vigor));
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
) {
    let parent_angles = Vector3::direction_to_euler_angles(&parent_normal);
    let angles_set = vec![Vector3::X() * plant_angle, Vector3::X() * -plant_angle, Vector3::Z() * plant_angle, Vector3::Z() * -plant_angle];

    for angle in angles_set {

    }
}


fn distribution(

) {

}

fn tropism(

) {

}

fn possible_collisions_volume(

) {

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
) {

    let directions = branch_prototypes.get_directions();
    let grav = gravity_res.gravity_dir * gravity_res.tropism_strength;


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
                        let rotation_axis = branch_parent_data.0.normal.cross(&Vector3::Y());
                        let rotation_angle = branch_parent_data.0.normal.angle_to(&Vector3::Y());
                        Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
                    };
                    (branch_parent_data.0.root_position, rotation_matrix)

                } else {
                    (Vector3::ZERO(), Matrix3::identity())
                }
            };


            if let Ok((mut branch_data, branch_growth_data, prototype_ref)) = branch_data_query.get_mut(id) {

                if branch_data.root_node.is_none() {continue;}
                let branch_age = branch_growth_data.physiological_age;


                if branch_data.parent_node.is_some() {
                    // update the root node position based on the position of the parent node (they're the same)
                    let root_position = {
                        if let Ok(parent_node) = branch_node_query.get(branch_data.parent_node.unwrap()) {
                            parent_node.position
                        } else {panic!("failed to get branch parent node")}
                    };
                    branch_data.root_position = root_position.transform(&parent_rotation_matrix) + parent_offset;
                }

                // update all node positions
                let node_pairs = get_nodes_and_connections_base_to_tip(&branch_node_connections_query, branch_data.root_node.unwrap());

                if node_pairs.len() == 0 {continue;}

                for i in 0..node_pairs.len() {
                    if let Ok(mut node_pair) = branch_node_query.get_many_mut(node_pairs[i]) {
                        
                        let segment_age = (branch_age - node_pair[1].phys_age).max(0.0);


                        let length = max_length.min(scale * segment_age); // i despise that I can't just min(a,b) for floats

                        let new_offset = directions[prototype_ref.0][i] * length;
                        let tropism_offset = ((grav * plant_tropism)/(segment_age + plant_tropism)) * (length / max_length);

                        node_pair[1].position = node_pair[0].position + new_offset;

                        if node_pair[1].position == Vector3::ZERO() && node_pairs[i][0] != branch_data.root_node.unwrap() {
                            panic!("Node positioned wrong, nodes: {:?}, {:?} \ndata: \nage: {:?} \nlength: {:?} \noffset: {:?} \nparent_pos: {:?}", node_pair[0], node_pair[1], segment_age, length, new_offset, node_pair[0].position);
                        }
                    }else {panic!("Could not get node pair, tried to get nodes: {:?}, {:?}", node_pairs[i][0], node_pairs[i][1])}
                }
            }
        }
    }
}