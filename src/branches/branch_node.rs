#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use super::super::maths::vector_three::Vector3;


#[derive(Component)]
pub struct BranchNodeTag;

#[derive(Component, Debug)]
pub struct BranchNodeData {
    pub position: Vector3,
    pub tropism_offset: Vector3,
    pub phys_age: f32,
    // node_type: Option<BranchNodeType>, // will only be used if the node is a special type, no need otherwise
    pub thickness: f32,
    pub thickening_factor: f32,
}

#[derive(Component, Debug)]
pub struct BranchNodeGrowthData {
    pub light_exposure: f32,
    pub growth_vigor: f32,
}

#[derive(Component, Debug)]
pub struct BranchNodeConnectionData {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

#[derive(Bundle)]
pub struct BranchNodeBundle {
    pub tag: BranchNodeTag,
    pub data: BranchNodeData,
    pub connections: BranchNodeConnectionData,
    pub growth_data: BranchNodeGrowthData,
}


impl Default for BranchNodeBundle {
    fn default() -> Self {
        BranchNodeBundle {
            tag: BranchNodeTag,
            data: BranchNodeData::default(),
            connections: BranchNodeConnectionData::default(),
            growth_data: BranchNodeGrowthData::default(),
        }
    }
}


impl Default for BranchNodeData {
    fn default() -> Self {
        BranchNodeData {
            position: Vector3::ZERO(),
            tropism_offset: Vector3::ZERO(),
            phys_age: 0.0,
            thickness: 0.0,
            thickening_factor: 0.0,
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


///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////// Node Sorting Stuff /////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// Returns a list of node ids fron the base to the tip, effectively creating a breadth first list
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

/// Returns a list of node ids fron the tip to the base, effectively creating a breadth first list then reversing it
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


/// Gets the terminal nodes from a branch
/// 
/// Does this by using an iterator that is only incremented when the node has no children, 
/// if the node has children, they are appended onto the list and then the node is removed from the list but the iterator isn't incremented
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

/// Returns a list of node ids and another list of how they are paired up
/// 
/// Does this by creating a breadth first search, but each time a new node is added, the index of the current node and the added node are added to the connections list
pub fn get_nodes_and_connections_base_to_tip(
    connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
) -> Vec<[Entity; 2]>{

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

    let mut node_pairs: Vec<[Entity; 2]> = Vec::new();

    for pair in connections {
        node_pairs.push([nodes[pair.0], nodes[pair.1]]);
    }

    node_pairs
}

/// The same as "get_nodes_and_connections_base_to_tip", but returns the position and branch width of the segments rather than their ids
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
            positions.push(node_data.position + node_data.tropism_offset);
            thicknesses.push(node_data.thickness);
        } else {
            panic!("oh god oh why")
        }
    }

    (positions, thicknesses, connections)
}


/// Returns a list of nodes on a layer of the tree, the first layer is layer 1,
/// Returns an empty vec if the layer is empty
pub fn get_nodes_on_layer(
    connections_query: &Query<&mut BranchNodeConnectionData, With<BranchNodeTag>>,
    root_node: Entity,
    target_layer: u32
) -> Vec<Entity> {
    let mut working_layer: Vec<Entity> = vec![root_node];
    let mut next_layer: Vec<Entity> = Vec::new();
    let mut layer = 1;

    loop {
        // break if on the correct layer
        if layer == target_layer {break;}
        if layer > target_layer{return Vec::new();}

        // add the nodes for the next layer
        for id in working_layer.iter() {
            if let Ok(node_connections) = connections_query.get(*id) {
                for child_node_id in node_connections.children.iter() {
                    next_layer.push(*child_node_id);
                }
            }
        }

        // swap layers and clear next
        working_layer = next_layer.clone();
        next_layer.clear();

        layer += 1;
    }
    working_layer
}



// /// returns only the non-terminal nodes from a branch
// pub fn get_non_terminal_nodes_base_to_tip(
//     connections_query: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
//     root_node: Entity,
// ) -> Vec<Entity> {
//     let mut list: Vec<Entity> = vec![root_node];

//     let mut i = 0;
//     loop {
//         if i >= list.len() {break;}
//         if let Ok(node_connections) = connections_query.get(list[i]) {
//             if node_connections.children.len() == 0 {
//                 list.swap_remove(i);
//                 continue;
//             }
//             for child_node_id in node_connections.children.iter() {
//                 list.push(*child_node_id);
//             }
//         }
//         i += 1;
//     }

//     list
// }