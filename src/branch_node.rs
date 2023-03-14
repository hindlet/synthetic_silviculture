#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::{
    vector_three::Vector3, graphics::general_graphics::Vertex,
};

#[derive(Component)]
pub struct BranchNodeTag;

#[derive(Component)]
pub struct BranchNodeData {
    pub position: Vector3,
    pub phys_age: f32,
    // node_type: Option<BranchNodeType>, // will only be used if the node is a special type, no need otherwise
    pub branch_length: f32, // length of the branch this node is on the end of, will figure out why it's used
    pub thickness: f32,
}

#[derive(Component)]
pub struct BranchNodeGrowthData {
    pub light_exposure: f32,
    pub growth_vigor: f32,
}

#[derive(Component)]
pub struct BranchNodeConnectionData {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

#[derive(Bundle)]
pub struct BranchNodeBundle {
    pub tag: BranchNodeTag,
    pub data: BranchNodeData,
    pub connections: BranchNodeConnectionData,
}


impl Default for BranchNodeBundle {
    fn default() -> Self {
        BranchNodeBundle {
            tag: BranchNodeTag,
            data: BranchNodeData::default(),
            connections: BranchNodeConnectionData::default()
        }
    }
}


impl Default for BranchNodeData {
    fn default() -> Self {
        BranchNodeData {
            position: Vector3::ZERO(),
            phys_age: 0.0,
            // node_type: None,
            branch_length: 0.0,
            thickness: 0.0,
        }
    }
}


impl Default for BranchNodeConnectionData {
    fn default() -> Self {
        BranchNodeConnectionData {
            parent: None,
            children: Vec::new(),
        }
    }
}

impl Default for BranchNodeGrowthData {
    fn default() -> Self {
        BranchNodeGrowthData {
            light_exposure: 0.0,
            growth_vigor: 0.0,
        }
    }
}




pub fn get_nodes_tip_to_base(
    connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
) -> Vec<Entity> {
    let mut list: Vec<Entity> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(node_connections) = connections_query.get(list[i]) {
            for child_node_id in node_connections.children.iter() {
                list.push(*child_node_id);
            }
        }
        i += 1;
    }

    list.reverse();

    list
}

pub fn get_nodes_base_to_tip(
    connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
) -> Vec<Entity> {
    let mut list: Vec<Entity> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(node_connections) = connections_query.get(list[i]) {
            for child_node_id in node_connections.children.iter() {
                list.push(*child_node_id);
            }
        }
        i += 1;
    }

    list
}


/// gets the terminal nodes from a branch
pub fn get_terminal_nodes(
    connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
) -> Vec<Entity> {

    let mut list: Vec<Entity> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if let Ok(node_connections) = connections_query.get(list[i]) {
            if node_connections.children.len() == 0 {
                i += 1;
                continue;
            }
            for child_node_id in node_connections.children.iter() {
                list.push(*child_node_id);
            }
            list.remove(i);
        }
        
    }

    list
}

pub fn get_node_data_and_connections_base_to_tip(
    connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    nodes_query: &Query<&BranchNodeData, With<BranchNodeTag>>,
    root_node: Entity,
) -> (Vec<Vector3>, Vec<f32>, Vec<(usize, usize)>) {

    let mut nodes: Vec<Entity> = vec![root_node];
    let mut connections: Vec<(usize, usize)> = Vec::new();

    let mut i = 0;
    loop {
        if i >= nodes.len() {break;}
        if let Ok(node_connections) = connections_query.get(nodes[i]) {
            for child_node_id in node_connections.children.iter() {
                nodes.push(*child_node_id);
                connections.push((i, nodes.len() - 1));
            }
        }
        i += 1;
    }

    let mut positions: Vec<Vector3> = Vec::new();
    let mut thicknesses: Vec<f32> = Vec::new();
    for id in nodes.iter() {
        if let Ok(node_data) = nodes_query.get(*id) {
            positions.push(node_data.position);
            thicknesses.push(node_data.thickness);
        } else {
            panic!("oh god oh why")
        }
    }
    
    (positions, thicknesses, connections)
}