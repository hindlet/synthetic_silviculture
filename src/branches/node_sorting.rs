#![allow(dead_code)]
use super::{
    branch_node::BranchNode,
    super::maths::vector_three::Vector3
};
use std::{mem::take, cell::RefCell, rc::Rc};


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// base to tip /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// Returns a list of node ids fron the base to the tip, effectively creating a breadth first list
pub fn nodes_base_to_tip(
    root_node: &Rc<RefCell<BranchNode>>
) -> Vec<Rc<RefCell<BranchNode>>> {
    
    let mut layers = get_node_layers(root_node);
    let mut list: Vec<Rc<RefCell<BranchNode>>> = Vec::new();

    for layer in layers.iter_mut() {
        list.append(layer);
    }

    list
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// tip to base /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

/// Returns a list of node fron the tip to the base, effectively creating a breadth first list then reversing it
pub fn nodes_tip_to_base(
    root_node: &Rc<RefCell<BranchNode>>
) -> Vec<Rc<RefCell<BranchNode>>> {
    let mut list = nodes_base_to_tip(root_node);
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
pub fn terminal_nodes(
    root_node: &Rc<RefCell<BranchNode>>,
) -> Vec<Rc<RefCell<BranchNode>>> {

    let nodes = nodes_base_to_tip(root_node);
    let mut list: Vec<Rc<RefCell<BranchNode>>> = Vec::new();

    for node_cell in nodes.iter() {
        if node_cell.as_ref().borrow().children.len() == 0 {
            list.push(Rc::clone(node_cell))
        }
    }

    list
}

/// returns the indices of the terminal nodes in the base-tip list
/// 
/// this is more useful than it sounds I promise
pub fn terminal_node_indices(
    root_node: &Rc<RefCell<BranchNode>>
) -> Vec<usize>{
    
    let nodes = nodes_base_to_tip(root_node);
    let mut out = Vec::new();

    for i in 0..nodes.len() {
        if nodes[i].as_ref().borrow().children.len() == 0{
            out.push(i)
        }
    }

    out

}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// connections /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// Returns a list of node ids and another list of how they are paired up
/// 
/// Does this by creating a breadth first search, but each time a new node is added, the index of the current node and the added node are added to the connections list
pub fn nodes_and_connections_base_to_tip(
    root_node: &Rc<RefCell<BranchNode>>,
) -> (Vec<Rc<RefCell<BranchNode>>>, Vec<(usize, usize)>) {

    let nodes = nodes_base_to_tip(root_node);

    let mut pairs = Vec::new();
    let mut list = vec![Rc::clone(&nodes[0])];

    for i in 0..nodes.len() {
        for child_cell in nodes[i].as_ref().borrow().children.iter() {
            list.push(Rc::clone(child_cell));
            pairs.push((i, list.len() - 1));
        }
    }

    (list, pairs)
}



/// The same as "get_nodes_and_connections_base_to_tip", but returns the position and branch width of the segments rather than their ids
pub fn get_node_data_and_connections_base_to_tip(
    root_node: &Rc<RefCell<BranchNode>>,
) -> (Vec<(Vector3, f32)>, Vec<(usize, usize)>) {

    let (list, pairs) = nodes_and_connections_base_to_tip(root_node);

    let mut out: Vec<(Vector3, f32)> = Vec::new();
    for node_cell in list.iter() {
        let node = node_cell.borrow();
        out.push((node.data.relative_position, node.data.radius));
    }

    (out, pairs)
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// layers //////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

/// Returns a list of nodes on a layer of the tree, the first layer is layer 1,
/// Returns an empty vec if the layer is empty
pub fn get_nodes_on_layer(
    root_node: &Rc<RefCell<BranchNode>>,
    target_layer: u32
) -> Vec<Rc<RefCell<BranchNode>>> {
    let mut working_layer: Vec<Rc<RefCell<BranchNode>>> = vec![Rc::clone(root_node)];
    let mut next_layer: Vec<Rc<RefCell<BranchNode>>> = Vec::new();
    let mut layer = 1;

    loop {
        // break if on the correct layer
        if layer == target_layer {break;}
        if layer > target_layer{return Vec::new();}

        for node_cell in working_layer {
            let node = node_cell.borrow();
            for child_node in node.children.iter() {
                next_layer.push(Rc::clone(child_node))
            }
        }

        // swap layers and clear next
        working_layer = take(&mut next_layer);

        layer += 1;
    }
    working_layer
}




pub fn get_node_layers(
    root_node: &Rc<RefCell<BranchNode>>,
)-> Vec<Vec<Rc<RefCell<BranchNode>>>>{
    let mut out = Vec::new();
    let mut working_layer: Vec<Rc<RefCell<BranchNode>>> = vec![Rc::clone(root_node)];
    let mut next_layer: Vec<Rc<RefCell<BranchNode>>> = Vec::new();

    loop {
        for node_cell in working_layer.iter() {
            for child_node in node_cell.borrow().children.iter() {
                next_layer.push(Rc::clone(child_node))
            }
        }

        // add working layer to out and swap next into working
        out.push(working_layer.clone());
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }
    out
}


