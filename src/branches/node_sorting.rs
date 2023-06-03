use super::{
    branch_node::BranchNode,
    super::maths::vector_three::Vector3
};
use std::mem::take;


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// base to tip /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// Returns a list of node ids fron the base to the tip, effectively creating a breadth first list
pub fn get_nodes_base_to_tip(
    root_node: &BranchNode
) -> Vec<&BranchNode> {
    let mut list: Vec<&BranchNode> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        for child_node in list[i].children.iter() {
            list.push(child_node)
        }
        i += 1;
    }

    list
}

pub fn get_mut_nodes_base_to_tip(
    root_node: &mut BranchNode
) -> Vec<&mut BranchNode> {
    let mut list: Vec<&mut BranchNode> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        for child_node in list[i].children.iter_mut() {
            list.push(child_node)
        }
        i += 1;
    }

    list
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// tip to base /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

/// Returns a list of node fron the tip to the base, effectively creating a breadth first list then reversing it
pub fn get_nodes_tip_to_base(
    root_node: &BranchNode
) -> Vec<&BranchNode> {
    let mut list = get_nodes_base_to_tip(root_node);
    list.reverse();
    list
}

/// Returns a list of mutable node fron the tip to the base, effectively creating a breadth first list then reversing it
pub fn get_mut_nodes_tip_to_base(
    root_node: &mut BranchNode
) -> Vec<&mut BranchNode> {
    let mut list = get_mut_nodes_base_to_tip(root_node);
    list.reverse();
    list
}



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// terminal ////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

/// Gets the terminal nodes from a branch
/// 
/// Does this by using an iterator that is only incremented when the node has no children, 
/// if the node has children, they are appended onto the list and then the node is removed from the list but the iterator isn't incremented
pub fn get_terminal_nodes(
    root_node: &BranchNode,
) -> Vec<&BranchNode> {

    let mut list: Vec<&BranchNode> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}

        let children = &list[i].children;
        
        if children.len() == 0 {
            i += 1;
            continue;
        }

        for child_node in children.iter() {
            list.push(child_node);
        }
        list.remove(i);
        
    }

    list
}

pub fn get_mut_terminal_nodes(
    root_node: &mut BranchNode,
) -> Vec<&mut BranchNode> {

    let mut list: Vec<&mut BranchNode> = vec![root_node];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}

        let children = &list[i].children;
        
        if children.len() == 0 {
            i += 1;
            continue;
        }

        for child_node in children.iter_mut() {
            list.push(child_node);
        }
        list.remove(i);
        
    }

    list
}

/// returns the indices of the terminal nodes in the base-tip list
/// 
/// this is more useful than it sounds I promise
pub fn get_terminal_node_indices(
    root_node: &BranchNode
) -> Vec<usize>{
    let mut nodes: Vec<&BranchNode> = vec![root_node];
    let mut out = Vec::new();

    let mut i = 0;
    loop {
        if i >= nodes.len() {break;}
        if nodes[i].children.len() == 0 {out.push(i)}
        for child_node in nodes[i].children.iter() {
            nodes.push(child_node)
        }
        i += 1;
    }

    out
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// connections /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// Returns a list of node ids and another list of how they are paired up
/// 
/// Does this by creating a breadth first search, but each time a new node is added, the index of the current node and the added node are added to the connections list
pub fn get_nodes_and_connections_base_to_tip(
    root_node: &BranchNode,
) -> (Vec<&BranchNode>, Vec<(usize, usize)>){

    let mut nodes: Vec<&BranchNode> = vec![root_node];
    let mut connections: Vec<(usize, usize)> = Vec::new();

    let mut i = 0;
    loop {
        if i >= nodes.len() {break;}
        for child_node in nodes[i].children.iter() {
            nodes.push(child_node);
            connections.push((i, nodes.len() - 1));
        }
        i += 1;
    }

    (nodes, connections)
}

/// Returns a list of mutable node references and another list of how they are paired up
/// 
/// Does this by creating a breadth first search, but each time a new node is added, the index of the current node and the added node are added to the connections list
pub fn get_mut_nodes_and_connections_base_to_tip(
    root_node: &mut BranchNode,
) -> (Vec<&mut BranchNode>, Vec<(usize, usize)>){

    let mut nodes: Vec<&mut BranchNode> = vec![root_node];
    let mut connections: Vec<(usize, usize)> = Vec::new();

    let mut i = 0;
    loop {
        if i >= nodes.len() {break;}
        for child_node in nodes[i].children.iter_mut() {
            nodes.push(child_node);
            connections.push((i, nodes.len() - 1));
        }
        i += 1;
    }

    (nodes, connections)
}



/// The same as "get_nodes_and_connections_base_to_tip", but returns the position and branch width of the segments rather than their ids
pub fn get_node_data_and_connections_base_to_tip(
    root_node: &BranchNode,
) -> (Vec<(Vector3, f32)>, Vec<(usize, usize)>) {

    let mut nodes: Vec<&BranchNode> = vec![root_node];
    let mut connections: Vec<(usize, usize)> = Vec::new();

    let mut i = 0;
    loop {
        if i >= nodes.len() {break;}
        for child_node in nodes[i].children.iter() {
            nodes.push(child_node);
            connections.push((i, nodes.len() - 1));
        }
        i += 1;
    }

    let mut out: Vec<(Vector3, f32)> = Vec::new();
    for node in nodes {
        out.push((node.data.relative_position, node.data.radius));
    }

    (out, connections)
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// layers //////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

/// Returns a list of nodes on a layer of the tree, the first layer is layer 1,
/// Returns an empty vec if the layer is empty
pub fn get_nodes_on_layer(
    root_node: &BranchNode,
    target_layer: u32
) -> Vec<&BranchNode> {
    let mut working_layer: Vec<&BranchNode> = vec![root_node];
    let mut next_layer: Vec<&BranchNode> = Vec::new();
    let mut layer = 1;

    loop {
        // break if on the correct layer
        if layer == target_layer {break;}
        if layer > target_layer{return Vec::new();}

        for node in working_layer {
            for child_node in node.children.iter() {
                next_layer.push(child_node)
            }
        }

        // swap layers and clear next
        working_layer = take(&mut next_layer);

        layer += 1;
    }
    working_layer
}

/// Returns a list of nodes on a layer of the tree, the first layer is layer 1,
/// Returns an empty vec if the layer is empty
pub fn get_mut_nodes_on_layer(
    root_node: &mut BranchNode,
    target_layer: u32
) -> Vec<&mut BranchNode> {
    let mut working_layer: Vec<&mut BranchNode> = vec![root_node];
    let mut next_layer: Vec<&mut BranchNode> = Vec::new();
    let mut layer = 1;

    loop {
        // break if on the correct layer
        if layer == target_layer {break;}
        if layer > target_layer{return Vec::new();}

        for node in working_layer {
            for child_node in node.children.iter_mut() {
                next_layer.push(child_node)
            }
        }

        // swap layers and clear next
        working_layer = take(&mut next_layer);

        layer += 1;
    }
    working_layer
}


pub fn get_node_layers(
    root_node: &BranchNode
)-> Vec<Vec<&BranchNode>>{
    let mut out = Vec::new();
    let mut working_layer: Vec<&BranchNode> = vec![root_node];
    let mut next_layer: Vec<&BranchNode> = Vec::new();

    loop {
        for node in working_layer {
            for child_node in node.children.iter() {
                next_layer.push(child_node)
            }
        }

        // add working layer to out and swap next into working
        out.push(working_layer);
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }
    out
}

pub fn get_mut_node_layers(
    root_node: &mut BranchNode
)-> Vec<Vec<&mut BranchNode>>{
    let mut out = Vec::new();
    let mut working_layer: Vec<&mut BranchNode> = vec![root_node];
    let mut next_layer: Vec<&mut BranchNode> = Vec::new();

    loop {
        for node in working_layer {
            for child_node in node.children.iter_mut() {
                next_layer.push(child_node)
            }
        }

        // add working layer to out and swap next into working
        out.push(working_layer);
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }
    out
}

