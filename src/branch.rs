#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use itertools::Itertools;
use super::{
    vector_three::Vector3,
    bounding_sphere::BoundingSphere,
    matrix_three::Matrix3,
    transform::Transform,
    branch_node::{BranchNodeData, BranchNodeTag, BranchNodeConnectionData, get_nodes_base_to_tip},
    branch_prototypes::{BranchPrototypeRef}, graphics::mesh::Mesh
};



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Default, Component)]
pub struct BranchTag;


#[derive(Debug, Component)]
pub struct BranchData {
    pub intersections_volume: f32,
    pub normal: Vector3,
    pub intersection_list: Vec<Entity>,
    pub root_node: Option<Entity>,
    pub parent_node: Option<Entity>, // a reference to the node on another branch that this branch started from
    pub root_position: Vector3,
}

#[derive(Debug, Component)]
pub struct BranchGrowthData {
    pub light_exposure: f32,
    pub growth_vigor: f32,
    pub growth_rate: f32,
    pub physiological_age: f32,
    pub layers: u32,
}

#[derive(Debug, Component)]
pub struct BranchBounds  {
    pub bounds: BoundingSphere
}


#[derive(Bundle)]
pub struct BranchBundle {
    pub tag: BranchTag,
    pub bounds: BranchBounds,
    pub data: BranchData,
    pub growth_data: BranchGrowthData,
    pub connections: BranchConnectionData,
    pub mesh: Mesh,
    pub prototype: BranchPrototypeRef,
}

#[derive(Debug, Component)]
pub struct BranchConnectionData {
    pub parent: Option<Entity>,
    pub children: (Option<Entity>, Option<Entity>),
}



///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Impl ///////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

impl BranchBundle {

    pub fn from_prototype() -> BranchBundle{
        BranchBundle::default()
    }
}

impl Default for BranchBundle {
    fn default() -> Self {
        BranchBundle {
            tag: BranchTag,
            bounds: BranchBounds::default(),
            data: BranchData::default(),
            growth_data: BranchGrowthData::default(),
            connections: BranchConnectionData::default(),
            mesh: Mesh::empty(),
            prototype: BranchPrototypeRef(0)
        }
    }
}

impl Default for BranchBounds {
    fn default() -> Self {
        Self {bounds: BoundingSphere::ZERO()}
    }
}

impl From<BoundingSphere> for BranchBounds {
    fn from(bounds: BoundingSphere) -> Self {
        Self {bounds}
    }
}


impl Default for BranchData {
    fn default() -> Self {
        BranchData {
            intersections_volume: 0.0,
            normal: Vector3::Y(),
            intersection_list: Vec::new(),
            root_node: None,
            parent_node: None,
            root_position: Vector3::ZERO(),
        }
    }
}

impl Default for BranchGrowthData {
    fn default() -> Self {
        BranchGrowthData {
            growth_vigor: 0.0,
            growth_rate: 0.0,
            light_exposure: 0.0,
            physiological_age: 0.0,
            layers: 1,
        }
    }
}

impl Default for BranchConnectionData {
    fn default() -> Self {
        BranchConnectionData {
            parent: None,
            children: (None, None),
        }
    }
}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Systems ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////



// updates branch bounds
pub fn update_branch_bounds(
    node_data: Query<&BranchNodeData, With<BranchNodeTag>>,
    nodes_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    mut branches_query: Query<(&mut BranchBounds, &BranchData), With<BranchTag>>
) {
    for (mut bounds, data) in &mut branches_query {
        if data.root_node.is_none() {continue;}

        let branch_rotation_matrix = {
            let mut rotation_axis = data.normal.cross(&Vector3::Y());
            rotation_axis.normalise();
            let rotation_angle = data.normal.angle_to(&Vector3::Y());
            Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
        };

        let mut node_positions: Vec<Vector3> = Vec::new();

        for id in get_nodes_base_to_tip(&nodes_connections_query, data.root_node.unwrap()) {
            if let Ok(node_data) = node_data.get(id) {
                node_positions.push(node_data.position.clone().transform(&branch_rotation_matrix));
            }
        }

        let mut new_bounds = 
            if node_positions.len() == 1 {
                BoundingSphere::new(node_positions[0], 0.01)
            }
            else {
                BoundingSphere::from_points(&node_positions)
            };
        
        new_bounds.centre += data.root_position;

        bounds.bounds = new_bounds;
    }
}

/// calculates branch intersection volumes
/// we use two querys so that we can get mutable borrows from both at once, you cannot do this with one query
pub fn calculate_branch_intersection_volumes(
    mut branch_query: Query<(&mut BranchData, &BranchBounds, Entity), With<BranchTag>>,
) {
    let mut intersection_lists: Vec<(Entity, BoundingSphere, Vec<Entity>)> = Vec::new();
    for (mut data, bounds, id) in branch_query.iter_mut() {
        data.intersections_volume = 0.0;
        let mut intersections = Vec::new();
        for id_other in data.intersection_list.iter() {
            intersections.push(*id_other);
        }
        intersection_lists.push((id, bounds.bounds.clone(), intersections));
    }

    for branch_one in intersection_lists {
        let mut volume = 0.0;
        for id in branch_one.2.iter() {
            if let Ok(mut branch_two) = branch_query.get_mut(*id) {
                let intersection = branch_one.1.get_intersection_volume(&branch_two.1.bounds);
                branch_two.0.intersections_volume += intersection;
                volume += intersection;
            }
        }
        if let Ok(mut branch) = branch_query.get_mut(branch_one.0) {
            branch.0.intersections_volume += volume;
        }
    }

}


pub fn calculate_branch_light_exposure(
    mut branches_query: Query<(&mut BranchGrowthData, &BranchData), With<BranchTag>>,
) {
    for mut data in branches_query.iter_mut() {
        data.0.light_exposure = (-data.1.intersections_volume).exp();
    }
}


///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////// Branch Sorting ////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


pub fn get_branch_parent_id(
    child_id: Entity,
    connections_query: &Query<&BranchConnectionData, With<BranchTag>>
) -> Option<Entity> {
    if let Ok(child_data) = connections_query.get(child_id) {
        return child_data.parent;
    }
    else {panic!("Failed to get parent branch")}
}


pub fn get_branches_tip_to_base(
    connections_query: &Query<&BranchConnectionData, With<BranchTag>>,
    root_branch: Entity,
) -> Vec<Entity> {
    let mut list: Vec<Entity> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(branch) = connections_query.get(list[i]) {
            if branch.children.0.is_some() {list.push(branch.children.0.unwrap())}
            if branch.children.1.is_some() {list.push(branch.children.1.unwrap())}
        }
        i += 1;
    }

    list.reverse();

    list
}

/// returns all branches from a root that can still have children
pub fn get_terminal_branches(
    connections_query: &Query<&mut BranchConnectionData, With<BranchTag>>,
    root_branch: Entity,
) -> Vec<Entity> {

    let mut list: Vec<Entity> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(branch_connections) = connections_query.get(list[i]) {
            if branch_connections.children.0.is_none() {
                i += 1;
                continue;
            }
            list.push(branch_connections.children.0.unwrap());
            if branch_connections.children.1.is_some() {
                list.push(branch_connections.children.1.unwrap());
            }
            list.remove(i);
        }
        
    }

    list
}

/// returns all non-terminal branches from a tree
pub fn get_non_terminal_branches(
    connections_query: &Query<&BranchConnectionData, With<BranchTag>>,
    root_branch: Entity
) -> Vec<Entity> {

    let mut list: Vec<Entity> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(branch) = connections_query.get(list[i]) {
            if branch.children.0.is_some() {list.push(branch.children.0.unwrap())}
            if branch.children.1.is_some() {list.push(branch.children.1.unwrap())}
            if branch.children.0.is_none() && branch.children.1.is_none() {list.swap_remove(i);}
            else {i += 1;}
        }
    }

    list
}

pub fn get_branches_base_to_tip(
    connections_query: &Query<&BranchConnectionData, With<BranchTag>>,
    root_branch: Entity,
) -> Vec<Entity> {
    let mut list: Vec<Entity> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(branch) = connections_query.get(list[i]) {
            if branch.children.0.is_some() {list.push(branch.children.0.unwrap())}
            if branch.children.1.is_some() {list.push(branch.children.1.unwrap())}
        }
        i += 1;
    }

    list
}


// calculates and returns the vigor of a branches children, assumes two children exist
pub fn get_children_vigor(
    branches_query: &Query<&mut BranchGrowthData, With<BranchTag>>,
    parent_vigor: f32,
    child_one: Entity,
    child_two: Entity,
    apical: f32,
) -> (f32, f32) {

    // get the light exposure from the two children
    let light_exp_one: f32 = {
        if let Ok(child_one_data) = branches_query.get(child_one) {child_one_data.light_exposure}
        else {0.0}
    };
    
    let light_exp_two: f32 = {
        if let Ok(child_two_data) = branches_query.get(child_two) {child_two_data.light_exposure}
        else {0.0}
    };

    // there is a main branch and a lateral branch, the main branch is the branch with the most light exposure
    // check which branch is main and use that to calculate using Vm = Vp * (apical * Qm) / (apical * Qm + 1-apical * Ql)
    // if they are the same then just split the vigor evenly
    
    if light_exp_one == light_exp_two {
        return (parent_vigor / 2.0, parent_vigor / 2.0);
    }
    else if light_exp_one > light_exp_two {
        let child_one_vigor = parent_vigor * ((apical * light_exp_one) / (apical * light_exp_one + (1.0-apical) * light_exp_two));
        return (child_one_vigor, parent_vigor-child_one_vigor);
    }
    else {
        let child_two_vigor = parent_vigor * ((apical * light_exp_two) / (apical * light_exp_two + (1.0-apical) * light_exp_one));
        return (parent_vigor - child_two_vigor, child_two_vigor);
    }
    
}



fn add_branch_child(
    connections_query: &mut Query<&mut BranchConnectionData, With<BranchTag>>,
    parent: Entity,
    new_child: Entity,
) -> bool {
    if let Ok(mut parent_connections) = connections_query.get_mut(parent) {
        if parent_connections.children.0.is_none() {
            parent_connections.children.0 = Some(new_child);
        }
        else if parent_connections.children.1.is_none() {
            parent_connections.children.1 = Some(new_child);
        } else {return false;}
    }

    if let Ok(mut child_connections) = connections_query.get_mut(new_child) {
        child_connections.parent = Some(parent);
    }

    true
}
