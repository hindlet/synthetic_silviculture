#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::general::*;
use crate::branch_prototypes::*;

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Default, Component)]
pub struct BranchTag;

pub enum BranchNodeType {
    Root,
    TerminalMain,
    TerminalLateral
}

#[derive(Default, Component)]
pub struct BranchNode {
    position: Vector3,
    age: Age,
    node_type: Option<BranchNodeType>, // will only be used if the node is a special type, no need otherwise
}

#[derive(Default, Component)]
pub struct BranchNodes {
    pub nodes: Vec<Entity>,
    pub connections: Vec<(usize, usize)>
}


#[derive(Default, Component)]
pub struct BranchData {
    pub growth_vigor: f32,
    pub intersections_volume: f32,
    pub light_exposure: f32,
    pub intersection_list: Vec<Entity>,
}



#[derive(Default, Bundle)]
pub struct BranchBundle {
    pub tag: BranchTag,
    pub bounds: BoundingSphere,
    pub nodes: BranchNodes,
    pub data: BranchData,
}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// impl /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

impl BranchBundle {
    pub fn new() -> Self{
        BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere::new(),
            nodes: BranchNodes {nodes: vec![], connections: vec![]},
            data: BranchData::new(),
        }

    }

    pub fn from_prototype() -> BranchBundle{
        BranchBundle::new()
    }
}


impl BranchData {
    pub fn new() -> Self {
        BranchData {
            growth_vigor: 0.0,
            intersections_volume: 0.0,
            light_exposure: 0.0,
            intersection_list: Vec::new(),
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
    mut branch_query_one: Query<(&mut BranchData, &BoundingSphere), With<BranchTag>>,
    mut branch_query_two: Query<(&mut BranchData, &BoundingSphere), With<BranchTag>>,
) {

    for (mut data, bounds) in branch_query_one.iter_mut() {
        for id in data.intersection_list.clone().iter() {
            if let Ok(mut other_branch) = branch_query_two.get_mut(*id) {
                let intersection_volume = bounds.get_intersection_volume(other_branch.1);
                data.intersections_volume += intersection_volume;
                other_branch.0.intersections_volume += intersection_volume;
            }
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

