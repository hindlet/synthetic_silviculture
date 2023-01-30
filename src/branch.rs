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
pub struct BranchNodeList {
    pub nodes: EntityList,
    pub connections: Vec<(usize, usize)>
}

#[derive(Default, Component)]
pub struct BranchIntersectionList {
    pub intersections: EntityList,
}


#[derive(Default, Component)]
pub struct IntersectionVolume {
    pub volume: f32
}


#[derive(Default, Component)]
pub struct LightExposure {
    pub light: f32,
}


#[derive(Default, Component)]
pub struct GrowthVigor {
    pub vigor: f32,
}


#[derive(Default, Bundle)]
pub struct BranchBundle {
    pub tag: BranchTag,
    pub bounds: BoundingSphere,
    pub nodes: BranchNodeList,
    pub intersection_volume: IntersectionVolume,
    pub intersections: BranchIntersectionList,
    pub light_exp: LightExposure,
}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// impl /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

impl BranchBundle {
    pub fn new() -> Self{
        BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere::new(),
            nodes: BranchNodeList {nodes: EntityList::new(), connections: vec![]},
            intersection_volume: IntersectionVolume::new(),
            intersections: BranchIntersectionList {intersections: EntityList::new()},
            light_exp: LightExposure::new(),
        }

    }

    pub fn from_prototype() -> BranchBundle{
        BranchBundle::new()
    }
}

impl GrowthVigor {
    pub fn new() -> Self {
        GrowthVigor {
            vigor: 0.0,
        }
    }
}

impl LightExposure {
    pub fn new() -> Self {
        LightExposure {
            light: 0.0,
        }
    }
}

impl IntersectionVolume {
    pub fn new() -> Self{
        IntersectionVolume {
            volume: 0.0,
        }
    }
}



///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// systems ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////



// updates branch bounds
pub fn update_branch_bounds(
    nodes_query: Query<&BranchNode>,
    mut branches_query: Query<(&mut BoundingSphere, &BranchNodeList), With<BranchTag>>
) {
    for (mut bounds, nodes) in &mut branches_query {

        let mut node_positions: Vec<Vector3> = Vec::new();

        for id in nodes.nodes.ids.iter() {
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
    mut branches_query_with_list: Query<(&mut IntersectionVolume, &BranchIntersectionList, &BoundingSphere), With<BranchTag>>,
    mut branches_query_no_list: Query<(&mut IntersectionVolume, &BoundingSphere), With<BranchTag>>,
) {

    for (mut volume, intersection_list, bounds) in branches_query_with_list.iter_mut() {
        for id in intersection_list.intersections.ids.iter() {
            if let Ok(mut other_branch) = branches_query_no_list.get_mut(*id) {
                let intersection_volume = bounds.get_intersection_volume(other_branch.1);
                volume.volume += intersection_volume;
                other_branch.0.volume += intersection_volume;
            }
        }
    }

}


pub fn calculate_branch_light_exposure (
    mut branches_query: Query<(&mut LightExposure, &IntersectionVolume), With<BranchTag>>,
) {
    for (mut exposure, volume) in branches_query.iter_mut() {
        exposure.light = (-volume.volume).exp();
    }
}

