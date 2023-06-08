#![allow(dead_code)]
use std::{mem::take, cell::RefCell, rc::Rc};
use super::{
    branch::Branch,
    super::{
        plants::plant::Plant,
        maths::bounding_sphere::BoundingSphere,
    },
};


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// global //////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// returns a base to tip pass through all the branches of each tree
pub fn get_all_branches(
    plants: &Vec<Plant>
) -> Vec<Rc<RefCell<Branch>>> {
    let mut out = Vec::new();

    for plant in plants.iter() {
        out.append(&mut branches_base_to_tip(&plant.root));
    }

    out
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// base to tip /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub fn branches_base_to_tip(
    root_branch: &Rc<RefCell<Branch>>,
) -> Vec<Rc<RefCell<Branch>>> {

    let mut layers = get_branch_layers(root_branch);
    let mut list: Vec<Rc<RefCell<Branch>>> = Vec::new();

    for layer in layers.iter_mut() {
        list.append(layer);
    }

    list
}

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// tip to base /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


pub fn branches_tip_to_base(
    root_branch: &Rc<RefCell<Branch>>,
) -> Vec<Rc<RefCell<Branch>>> {
    let mut list = branches_base_to_tip(root_branch);
    list.reverse();

    list
}



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// terminal ////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// returns all branches from a root that can still have children
pub fn branches_terminal(
    root_branch: &Rc<RefCell<Branch>>,
) -> Vec<Rc<RefCell<Branch>>> {

    let branches = branches_base_to_tip(root_branch);
    let mut list: Vec<Rc<RefCell<Branch>>> = Vec::new();

    for branch_cell in branches.iter() {
        if branch_cell.as_ref().borrow().children.0.is_none() {
            list.push(Rc::clone(branch_cell))
        }
    }

    list
}

pub fn branches_terminal_with_index(
    root_branch: &Rc<RefCell<Branch>>
) -> Vec<(Rc<RefCell<Branch>>, usize)> {

    let mut out = Vec::new();
    let mut working_layer: Vec<Rc<RefCell<Branch>>> = vec![Rc::clone(root_branch)];
    let mut next_layer: Vec<Rc<RefCell<Branch>>> = Vec::new();


    loop {
        for i in 0..working_layer.len() {

            let branch = working_layer[i].borrow();

            if branch.children.0.is_some() {
                next_layer.push(Rc::clone(&branch.children.0.as_ref().unwrap()))
            } else {
                out.push((Rc::clone(&working_layer[i]), i))
            }
            if branch.children.1.is_some() {next_layer.push(Rc::clone(&branch.children.1.as_ref().unwrap()))}

        }

        // add working layer to out and swap next into working
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }

    out
}



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// non terminal ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// returns all non-terminal branches from a tree in no particular order
pub fn non_terminal_branches(
    root_branch: &Rc<RefCell<Branch>>
) -> Vec<Rc<RefCell<Branch>>> {

    let branches = branches_base_to_tip(root_branch);
    let mut list = Vec::new();

    for branch_cell in branches.iter() {
        if branch_cell.as_ref().borrow().children.0.is_some() {
            list.push(Rc::clone(branch_cell));
        }
    }

    list
}



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// data ////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


pub fn get_branch_bounds(
    root_branch: &Rc<RefCell<Branch>>
) -> Vec<BoundingSphere> {
    let mut out = Vec::new();

    for branch in branches_base_to_tip(root_branch) {
        out.push(branch.borrow().bounds.clone());
    }

    out
}

/// returns a list of all mutable branches needing mesh updates and the time since the update was required
pub fn get_branch_mesh_update_times(
    plants: &Vec<Plant>
) -> Vec<(f32, Rc<RefCell<Branch>>)> {
    let mut out = Vec::new();

    for branch in get_all_branches(plants) {
        if let Some(called) = branch.borrow().needs_mesh_update {
            out.push((called.elapsed().as_secs_f32(), Rc::clone(&branch)));
        }
    }

    out
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// layers //////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub fn get_branch_layers(
    root_branch: &Rc<RefCell<Branch>>
)-> Vec<Vec<Rc<RefCell<Branch>>>>{
    let mut out = Vec::new();
    let mut working_layer: Vec<Rc<RefCell<Branch>>> = vec![Rc::clone(root_branch)];
    let mut next_layer: Vec<Rc<RefCell<Branch>>> = Vec::new();

    loop {
        for branch_cell in working_layer.iter() {
            let branch = branch_cell.borrow();
            if branch.children.0.is_some() {next_layer.push(Rc::clone(&branch.children.0.as_ref().unwrap()))}
            if branch.children.1.is_some() {next_layer.push(Rc::clone(&branch.children.1.as_ref().unwrap()))}
        }

        // add working layer to out and swap next into working
        out.push(working_layer.clone());
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }
    out
}