#![allow(dead_code, unused_variables, unused_imports)]

use bevy_ecs::prelude::*;
use itertools::*;
use plotters::data;
use crate::{
    branch::{BranchTag, BranchData, get_branches_base_to_tip, get_branches_tip_to_base, get_non_terminal_branches, get_children_vigor, BranchConnectionData, BranchGrowthData, BranchBounds},
    vector_three::Vector3,
    bounding_box::BoundingBox,
    bounding_sphere::BoundingSphere,
    environment::PhysicalAgeStep,
};

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[derive(Default, Component)]
pub struct PlantTag;



#[derive(Component)]
pub struct PlantData {
    pub position: Vector3,
    pub intersection_list: Vec<Entity>,
    pub age: f32,
    pub root_node: Option<Entity>,
}

#[derive(Component)]
pub struct PlantGrowthControlFactors {
    pub max_age: f32,
    pub max_vigor: f32,
    pub min_vigor: f32,
    pub apical_control: f32, // range 0..1
    pub orientation_angle: f32,
    pub tropism_angle_weight: f32, // range 0..1
    pub growth_rate: f32,
    pub max_branch_segment_length: f32,
    pub branch_segment_length_scaling_coef: f32,
    pub tropism_time_control: f32,
}

#[derive(Component)]
pub struct PlantBounds {
    pub bounds: BoundingBox,
}


#[derive(Bundle)]
pub struct PlantBundle {
    pub tag: PlantTag,
    pub bounds: PlantBounds,
    pub data: PlantData,
    pub growth_factors: PlantGrowthControlFactors,
}

#[derive(Resource)]
pub struct PlantDeathRate {
    pub v_max_decrease: f32,
}


///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////// impls /////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

impl PlantDeathRate {
    pub fn new(death_rate: f32) -> Self {
        PlantDeathRate {
            v_max_decrease: death_rate,
        }
    }
}


impl Default for PlantBundle {
    fn default() -> Self {
        PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::default(),
            data: PlantData::default(),
            growth_factors: PlantGrowthControlFactors::default(),
        }
    }
}

impl Default for PlantData {
    fn default() -> Self {
        PlantData {
            root_node: None,
            position: Vector3::ZERO(),
            intersection_list: Vec::new(),
            age: 0.0,
        }
    }
}

impl Default for PlantBounds {
    fn default() -> Self {
        PlantBounds {
            bounds: BoundingBox::ZERO(),
        }
    }
}

impl From<BoundingBox> for PlantBounds {
    fn from(bounds: BoundingBox) -> Self {
        Self {
            bounds
        }
    }
}

impl Default for PlantGrowthControlFactors {
    fn default() -> Self {
        PlantGrowthControlFactors {
            max_vigor: 0.0,
            min_vigor: 0.0,
            max_age: 0.0,
            apical_control: 0.5,
            orientation_angle: 0.0,
            tropism_angle_weight: 0.5,
            growth_rate: 1.0,
            max_branch_segment_length: 1.0,
            branch_segment_length_scaling_coef: 1.0,
            tropism_time_control: 1.0,
        }
    }
}   




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// systems ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


// must be called after updating all the branch bounds
pub fn update_plant_bounds(
    branch_bounds_query: Query<&BranchBounds, With<BranchTag>>,
    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
    mut plants_query: Query<(&mut PlantBounds, &PlantData), With<PlantTag>>
) {
    for (mut bounds, plant_data) in &mut plants_query {
        if plant_data.root_node.is_none() {continue;}

        let mut branch_bounds: Vec<BoundingSphere> = Vec::new();

        
        for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {
            if let Ok(bounds) = branch_bounds_query.get(id) {
                branch_bounds.push(bounds.bounds.clone());
            }
        }

        let new_bounds = BoundingBox::from_spheres(&branch_bounds);
        bounds.bounds = new_bounds;
    }
}

// this will calculate all the plant intersections, it will not contain any repeated intersect
pub fn update_plant_intersections(
    mut plants_query: Query<(&PlantBounds, &mut PlantData, Entity), With<PlantTag>>,
) {
    // reset all plant intersection lists
    for (bounds, mut data, id) in &mut plants_query {
        data.intersection_list = Vec::new();
    }
    // check all plant intersection options
    let mut combinations = plants_query.iter_combinations_mut();
    while let Some([mut plant_one, plant_two]) = combinations.fetch_next() {
        if plant_one.0.bounds.is_intersecting_box(&plant_two.0.bounds) {
            plant_one.1.intersection_list.push(plant_two.2);
        }
    }
}


/// this relies on the fact that our plant intersections will not contain any repeats,
/// if they did the branches would end up with double the intersection volumes they are meant to
pub fn update_branch_intersections(
    plants_query: Query<&PlantData, With<PlantTag>>,
    mut branch_query: Query<(&BranchBounds, &mut BranchData), With<BranchTag>>,
    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
) {
    // loop through each plant
    for plant_data in plants_query.iter() {
        if plant_data.root_node.is_none() {continue;}
        
        // loop through intersections
        for other_plant_id in plant_data.intersection_list.iter() {

            // get a list of the bounds of the other plants branches
            let mut other_plant_branch_bounds: Vec<(BoundingSphere, Entity)> = vec![];

            // loop through all the branches we could intersect with and add them to a list
            if let Ok(other_plant) = plants_query.get(*other_plant_id) {
                if other_plant.root_node.is_none() {continue;}
                for id in get_branches_base_to_tip(&branch_connections_query, other_plant.root_node.unwrap()) {
                    if let Ok(branch) = &branch_query.get(id) {
                        other_plant_branch_bounds.push((branch.0.bounds.clone(), id));
                    }
                }
            }

            // loop through each of our branches
            for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {
                if let Ok(mut branch) = branch_query.get_mut(id) {
                    // reset the branches intersections list and volume
                    branch.1.intersection_list = Vec::new();
                    branch.1.intersections_volume = 0.0;
                    // check if the branches intersect, if so, add the second branch id to the first's list
                    for other_bounds in other_plant_branch_bounds.iter() {
                        if branch.0.bounds.is_intersecting_sphere(&other_bounds.0) {
                            branch.1.intersection_list.push(other_bounds.1);
                        }
                    }
                }
            } 

            // check through our own branches for collissions
            // I don't like this code but i had to fight the borrow checker
            for combination in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()).iter().combinations(2) {
                let other_data: BoundingSphere;
                if let Ok(branch_two) = branch_query.get(*combination[1]){
                    other_data = branch_two.0.bounds.clone();
                } else {panic!("Fuck balls shit fuck balls")};
                if let Ok(mut branch_one) = branch_query.get_mut(*combination[0]){
                    if branch_one.0.bounds.is_intersecting_sphere(&other_data) {
                        branch_one.1.intersection_list.push(*combination[1]);
                    }
                };
                
            }
        }

    }
}

/// steps the ages of all plants by the phyical age step
/// also adjusts max vigor where appropriate
pub fn step_plant_age(
    mut plant_query: Query<(&mut PlantData, &mut PlantGrowthControlFactors), With<PlantTag>>,
    timestep: Res<PhysicalAgeStep>,
    descrease_rate: Res<PlantDeathRate>
) {
    for (mut plant_data, mut plant_growth_data) in plant_query.iter_mut() {
        plant_data.age += timestep.step;
        if plant_data.age > plant_growth_data.max_age {
            plant_growth_data.max_vigor = 0.0_f32.max(plant_growth_data.max_vigor - descrease_rate.v_max_decrease);
        }
    }
}

/// takes data from the branches and distributes it, we do a tip to base pass and sum light exposure at branching points
/// after this we use a helper function to distribute growth vigor up the plant
/// this means that branches closer to the root have a higher growth vigor than those further away
pub fn calculate_growth_vigor(
    plant_query: Query<(&PlantData, &PlantGrowthControlFactors), With<PlantTag>>,
    mut branch_query: Query<&mut BranchGrowthData, With<BranchTag>>,
    branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>
) {
    for plant_data in plant_query.iter() {

        if plant_data.0.root_node.is_none() {continue;}
        
        // reset light exposure in all none-tip branches
        for id in get_non_terminal_branches(&branch_connections_query, plant_data.0.root_node.unwrap()) {
            if let Ok(mut branch_data) = branch_query.get_mut(id){
                branch_data.light_exposure = 0.0;
            }
        }

        // sum up light exposure at branching_points
        for id in get_branches_tip_to_base(&branch_connections_query, plant_data.0.root_node.unwrap()) {
            #[allow(unused_assignments)]
            let mut light_exposure = 0.0;
            if let Ok(branch_data) = branch_query.get(id) {
                light_exposure = branch_data.light_exposure;
            } else {panic!("Fuck shit balls")}
            if let Ok(branch_connections) = branch_connections_query.get(id) {
                if branch_connections.parent.is_none() {continue;}
                if let Ok(mut parent_data) = branch_query.get_mut(branch_connections.parent.unwrap()) {
                    parent_data.light_exposure += light_exposure;
                }
            }
        }

        if let Ok(mut root_data) = branch_query.get_mut(plant_data.0.root_node.unwrap()) {
            root_data.growth_vigor = root_data.light_exposure.max(plant_data.1.max_vigor);
        }
        // distribute vigor to branches
        for id in get_branches_base_to_tip(&branch_connections_query, plant_data.0.root_node.unwrap()) {
            #[allow(unused_assignments)]
            let mut vigor = 0.0;

            if let Ok(parent_data) = branch_query.get(id) {
                vigor = parent_data.growth_vigor;
            } else {panic!("Fuck shit balls")}

            if let Ok(parent_connections) = branch_connections_query.get(id) {
                
                if parent_connections.children.1.is_none() {
                    if parent_connections.children.0.is_none() {continue;}
                    if let Ok(mut only_child) = branch_query.get_mut(parent_connections.children.0.unwrap()) {
                        only_child.growth_vigor = vigor;
                        continue;
                    } 
                }

                let vigor_distribution = get_children_vigor(
                    &branch_query, vigor, 
                    parent_connections.children.0.unwrap(), parent_connections.children.1.unwrap(), plant_data.1.apical_control);

                if let Ok(mut children) = branch_query.get_many_mut([parent_connections.children.0.unwrap(), parent_connections.children.1.unwrap()]) {
                    children[0].growth_vigor = vigor_distribution.0;
                    children[1].growth_vigor = vigor_distribution.1;
                }
            
            }   
        }

    }
}

