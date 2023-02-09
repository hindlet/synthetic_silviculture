#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use itertools::Itertools;
use crate::{
    vector3::Vector3,
    bounding_sphere::BoundingSphere,
    
};

/// this is the code for branches
/// a branch can connect have two other branches connect to it in a plant


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Default, Component)]
pub struct BranchTag;

#[derive(Component)]
pub struct BranchNode {
    position: Vector3,
    age: f32,
    // node_type: Option<BranchNodeType>, // will only be used if the node is a special type, no need otherwise
    branch_length: f32, // length of the branch this node is on the end of, will figure out why it's used
    thickness: f32, 
}

#[derive(Component)]
pub struct BranchNodes {
    pub nodes: Vec<Entity>,
    pub connections: Vec<(usize, usize)>
}


#[derive(Component)]
pub struct BranchData {
    pub growth_vigor: f32,
    pub intersections_volume: f32,
    pub light_exposure: f32,
    pub intersection_list: Vec<Entity>,
}

#[derive(Component)]
pub struct BranchConnectionData {
    pub parent: Option<Entity>,
    pub children: (Option<Entity>, Option<Entity>),
}



#[derive(Bundle)]
pub struct BranchBundle {
    pub tag: BranchTag,
    pub bounds: BoundingSphere,
    pub nodes: BranchNodes,
    pub data: BranchData,
    pub connections: BranchConnectionData,
}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// impl /////////////////////////////////////////////
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
            bounds: BoundingSphere::new(),
            nodes: BranchNodes::default(),
            data: BranchData::default(),
            connections: BranchConnectionData::default(),
        }
    }
}


impl Default for BranchData {
    fn default() -> Self {
        BranchData {
            growth_vigor: 0.0,
            intersections_volume: 0.0,
            light_exposure: 0.0,
            intersection_list: Vec::new(),
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

impl BranchNode {
    pub fn new() -> Self {
        BranchNode {
            position: Vector3::new(),
            age: 0.0,
            // node_type: None,
            branch_length: 0.0,
            thickness: 1.0,
        }
    }
}

impl Default for BranchNodes {
    fn default() -> Self {
        BranchNodes {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }
}


///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// systems ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////



// updates branch bounds
pub fn update_branch_bounds(
    nodes_query: Query<&BranchNode>,
    mut branches_query: Query<(&mut BoundingSphere, &BranchNodes), With<BranchTag>>
) {
    for (mut bounds, nodes) in &mut branches_query {

        let mut node_positions: Vec<Vector3> = Vec::new();

        for id in nodes.nodes.iter() {
            if let Ok(branch_node) = nodes_query.get(*id) {
                node_positions.push(branch_node.position);
            }
        }

        let new_bounds = BoundingSphere::from_points(&node_positions);
        bounds.set_to(&new_bounds);
    }
}

/// calculates branch intersection volumes
/// we use two querys so that we can get mutable borrows from both at once, you cannot do this with one query
pub fn calculate_branch_intersection_volumes (
    mut branch_query: Query<(&mut BranchData, &BoundingSphere, Entity), With<BranchTag>>,
) {
    let mut intersection_lists: Vec<(Entity, BoundingSphere, Vec<Entity>)> = Vec::new();
    for (mut data, bounds, id) in branch_query.iter_mut() {
        data.intersections_volume = 0.0;
        let mut intersections = Vec::new();
        for id_other in data.intersection_list.iter() {
            intersections.push(*id_other);
        }
        intersection_lists.push((id, bounds.clone(), intersections));
    }

    for branch_one in intersection_lists {
        let mut volume = 0.0;
        for id in branch_one.2.iter() {
            if let Ok(mut branch_two) = branch_query.get_mut(*id) {
                let intersection = branch_one.1.get_intersection_volume(branch_two.1);
                branch_two.0.intersections_volume += intersection;
                volume += intersection;
            }
        }
        if let Ok(mut branch) = branch_query.get_mut(branch_one.0) {
            branch.0.intersections_volume += volume;
        }
    }

}


pub fn calculate_branch_light_exposure (
    mut branches_query: Query<&mut BranchData, With<BranchTag>>,
) {
    for mut data in branches_query.iter_mut() {
        data.light_exposure = (-data.intersections_volume).exp();
    }
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
    branches_query: &Query<&mut BranchData, With<BranchTag>>,
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
