#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use itertools::*;
use crate::general::*;
use crate::branch::*;

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Default, Component)]
pub struct PlantTag;

#[derive(Default, Component)]
pub struct BranchList {
    pub branches: EntityList,
}

#[derive(Default, Component)]
pub struct PlantIntersectionList {
    pub intersections: EntityList,
}

impl PlantIntersectionList {
    pub fn set_zero(&mut self) {
        self.intersections = EntityList::new();
    }
}


#[derive(Default, Bundle)]
pub struct PlantBundle {
    pub tag: PlantTag,
    pub position: Vector3,
    pub branches: BranchList,
    pub bounds: BoundingBox,
    pub intersections: PlantIntersectionList,
}


///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////// plant /////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


impl PlantBundle {
    pub fn new() -> Self {
        PlantBundle {
            tag: PlantTag,
            position: Vector3::new(),
            branches: BranchList {branches: EntityList::new()},
            bounds: BoundingBox::new(),
            intersections: PlantIntersectionList {intersections: EntityList::new()},
        }
    }
}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// systems ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


// must be called after updating all the branch bounds
pub fn update_plant_bounds(
    branches_query: Query<&BoundingSphere, With<BranchTag>>,
    mut plants_query: Query<(&mut BoundingBox, &BranchList), With<PlantTag>>
) {
    for (mut bounds, branch_ids) in &mut plants_query {

        let mut branch_bounds: Vec<BoundingSphere> = Vec::new();

        for id in branch_ids.branches.ids.iter() {
            if let Ok(bounds) = branches_query.get(*id) {
                branch_bounds.push(bounds.clone());
            }
        }

        let new_bounds = BoundingBox::from_spheres(&branch_bounds);
        bounds.set_to(&new_bounds);
    }
}

// this will calculate all the plant intersections, it will not contain any repeated intersect
pub fn update_plant_intersections(
    mut plants_query: Query<(&BoundingBox, &mut PlantIntersectionList, Entity), With<PlantTag>>,
) {
    // reset all plant intersection lists
    for (bounds, mut intersections, id) in &mut plants_query {
        intersections.set_zero();
    }
    // check all plant intersection options
    let mut combinations = plants_query.iter_combinations_mut();
    while let Some([mut plant_one, plant_two]) = combinations.fetch_next() {
        if plant_one.0.is_intersecting_box(&plant_two.0) {
            plant_one.1.intersections.ids.push(plant_two.2);
        }
    }
}


/// this relies on the fact that our plant intersections will not contain any repeats,
/// if they did the branches would end up with double the intersection volumes they are meant to
pub fn update_branch_intersections(
    plants_query: Query<(&PlantIntersectionList, &mut BranchList), With<PlantTag>>,
    mut branch_query: Query<(&BoundingSphere, &mut BranchIntersectionList, &mut IntersectionVolume), With<BranchTag>>,
) {
    // loop through each plant
    for (intersection_ids, branch_list) in plants_query.iter() {
        
        // loop through intersections
        for other_plant_id in intersection_ids.intersections.ids.iter() {

            // get a list of the bounds of the other plants branches
            let mut other_plant_branch_bounds: Vec<(BoundingSphere, Entity)> = vec![];

            // loop through all the branches we could intersect with and add them to a list
            if let Ok(other_plant) = plants_query.get(*other_plant_id) {
                for id in other_plant.1.branches.ids.iter() {
                    if let Ok(branch) = &branch_query.get(*id) {
                        other_plant_branch_bounds.push((branch.0.clone(), *id));
                    }
                }
            }

            // loop through each of our branches
            for id in branch_list.branches.ids.iter() {
                if let Ok(mut branch) = branch_query.get_mut(*id) {
                    // reset the branches intersections list and volume
                    branch.1.intersections.ids = vec![];
                    branch.2.volume = 0.0;
                    // check if the branches intersect, if so, add the second branch id to the first's list
                    for other_bounds in other_plant_branch_bounds.iter() {
                        if branch.0.is_intersecting_sphere(&other_bounds.0) {
                            branch.1.intersections.ids.push(other_bounds.1);
                        }
                    }
                }
            } 


            for combination in branch_list.branches.ids.iter().combinations(2) {
                println!("{:?}", combination);
            }
        }

        

    }
}



