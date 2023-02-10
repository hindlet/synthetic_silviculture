#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::{
    transform::Transform,    
};

#[derive(Component)]
pub struct BranchNodeTag;

#[derive(Component)]
pub struct BranchNode {
    transform: Transform,
    phys_age: f32,
    // node_type: Option<BranchNodeType>, // will only be used if the node is a special type, no need otherwise
    branch_length: f32, // length of the branch this node is on the end of, will figure out why it's used
    thickness: f32, 
}

#[derive(Component)]
pub struct BranchNodeConnectionData {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}


impl Default for BranchNode {
    fn default() -> Self {
        BranchNode {
            transform: Transform::default(),
            phys_age: 0.0,
            // node_type: None,
            branch_length: 0.0,
            thickness: 0.0,
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