use std::mem::take;

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
pub fn get_all_branches<T>(
    plants: T
) -> Vec<&'static Branch>
    where
        T: IntoIterator<Item = Plant>
{
    let mut out = Vec::new();

    for plant in plants.iter() {
        out.append(&mut get_branches_base_to_tip(&plant.root));
    }

    out
}

/// returns a base to tip pass through all the mutable branches of each tree
pub fn get_mut_all_branches<T>(
    plants: T
) -> Vec<&'static mut Branch>
    where
        T: IntoIterator<Item = Plant>
{
    let mut out = Vec::new();

    for plant in plants.iter() {
        out.append(&mut get_mut_branches_base_to_tip(&mut plant.root));
    }

    out
}

/// returns a list of all mutable branches needing mesh updates and the time since the update was required
pub fn get_branch_mesh_update_times<T>(
    plants: T
) -> Vec<(f32, &'static mut Branch)>
    where
        T: IntoIterator<Item = Plant>
{
    let mut out = Vec::new();

    for branch in get_mut_all_branches(plants) {
        if let Some(called) = branch.needs_mesh_update {
            out.push((called.elapsed().as_secs_f32(), branch));
        }
    }

    out
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// base to tip /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub fn get_branches_base_to_tip(
    root_branch: &Branch,
) -> Vec<&Branch> {
    let mut list: Vec<&Branch> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if list[i].children.0.is_some() {list.push(&list[i].children.0.unwrap())}
        if list[i].children.1.is_some() {list.push(&list[i].children.1.unwrap())}
        i += 1;
    }

    list
}

pub fn get_mut_branches_base_to_tip(
    root_branch: &mut Branch,
) -> Vec<&mut Branch> {
    let mut list: Vec<&mut Branch> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}
        if list[i].children.0.is_some() {list.push(&mut list[i].children.0.unwrap())}
        if list[i].children.1.is_some() {list.push(&mut list[i].children.1.unwrap())}
        i += 1;
    }

    list
}

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// tip to base /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


pub fn get_branches_tip_to_base(
    root_branch: &Branch,
) -> Vec<&Branch> {
    let mut list = get_branches_base_to_tip(root_branch);
    list.reverse();

    list
}

pub fn get_mut_branches_tip_to_base(
    root_branch: &mut Branch,
) -> Vec<&mut Branch> {
    let mut list = get_mut_branches_base_to_tip(root_branch);
    list.reverse();

    list
}



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// terminal ////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


/// returns all branches from a root that can still have children
pub fn get_terminal_branches(
    root_branch: &Branch,
) -> Vec<&Branch> {

    let mut list: Vec<&Branch> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}

        if list[i].children.0.is_some() {
            list.push(&list[i].children.0.unwrap())
        }
        else {
            i += 1;
            continue;
        }

        if list[i].children.1.is_some() {list.push(&list[i].children.1.unwrap())}
        list.remove(i)
    }

    list
}

pub fn get_mut_terminal_branches(
    root_branch: &mut Branch,
) -> Vec<&mut Branch> {

    let mut list: Vec<&mut Branch> = vec![root_branch];

    let mut i = 0;
    loop {
        if i >= list.len() {break;}

        if list[i].children.0.is_some() {
            list.push(&mut list[i].children.0.unwrap())
        }
        else {
            i += 1;
            continue;
        }

        if list[i].children.1.is_some() {list.push(&mut list[i].children.1.unwrap())}
        list.remove(i)
    }

    list
}


pub fn get_mut_terminal_branches_with_index(
    root_branch: &mut Branch,
) -> Vec<(&mut Branch, usize)> {

    let mut out = Vec::new();
    let mut working_layer: Vec<&mut Branch> = vec![root_branch];
    let mut next_layer: Vec<&mut Branch> = Vec::new();


    loop {
        for i in 0..working_layer.len() {

            if working_layer[i].children.0.is_some() {
                next_layer.push(&mut working_layer[i].children.0.unwrap())
            } else {
                out.push((working_layer[i], i))
            }
            if working_layer[i].children.1.is_some() {next_layer.push(&mut working_layer[i].children.1.unwrap())}

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


/// returns all non-terminal branches from a tree
pub fn get_non_terminal_branches(
    root_branch: &Branch
) -> Vec<&Branch> {

    let mut list: Vec<&Branch> = vec![root_branch];

    let mut i = 0;
    let mut full = (false, false);
    loop {
        if i >= list.len() {break;}

        full = (false, false);
        if list[i].children.0.is_some() {
            list.push(&list[i].children.0.unwrap());
            full.0 = true;
        }
        if list[i].children.1.is_some() {
            list.push(&list[i].children.1.unwrap());
            full.1 = true;
        }

        if !full.0 && !full.1 {
            list.swap_remove(i);
        } else {i += 1;}
    }

    list
}

/// returns all non-terminal branches from a tree
pub fn get_mut_non_terminal_branches(
    root_branch: &mut Branch
) -> Vec<&mut Branch> {

    let mut list: Vec<&mut Branch> = vec![root_branch];

    let mut i = 0;
    let mut full = (false, false);
    loop {
        if i >= list.len() {break;}

        full = (false, false);
        if list[i].children.0.is_some() {
            list.push(&mut list[i].children.0.unwrap());
            full.0 = true;
        }
        if list[i].children.1.is_some() {
            list.push(&mut list[i].children.1.unwrap());
            full.1 = true;
        }

        if !full.0 && !full.1 {
            list.swap_remove(i);
        } else {i += 1;}
    }

    list
}



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// data ////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


pub fn get_branch_bounds(
    root_branch: &Branch
) -> Vec<&'static BoundingSphere> {
    let mut out = Vec::new();

    for branch in get_branches_base_to_tip(root_branch) {
        out.push(&branch.bounds);
    }

    out
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// layers //////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub fn get_branch_layers(
    root_branch: &Branch
)-> Vec<Vec<&Branch>>{
    let mut out = Vec::new();
    let mut working_layer: Vec<&Branch> = vec![root_branch];
    let mut next_layer: Vec<&Branch> = Vec::new();

    loop {
        for branch in working_layer {
            if branch.children.0.is_some() {next_layer.push(&branch.children.0.unwrap())}
            if branch.children.1.is_some() {next_layer.push(&branch.children.1.unwrap())}
        }

        // add working layer to out and swap next into working
        out.push(working_layer);
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }
    out
}

pub fn get_mut_branch_layers(
    root_branch: &mut Branch
)-> Vec<Vec<&mut Branch>>{
    let mut out = Vec::new();
    let mut working_layer: Vec<&mut Branch> = vec![root_branch];
    let mut next_layer: Vec<&mut Branch> = Vec::new();

    loop {
        for branch in working_layer {
            if branch.children.0.is_some() {next_layer.push(&mut branch.children.0.unwrap())}
            if branch.children.1.is_some() {next_layer.push(&mut branch.children.1.unwrap())}
        }

        // add working layer to out and swap next into working
        out.push(working_layer);
        working_layer = take(&mut next_layer);

        if working_layer.len() == 0 {break;}
    }
    out
}