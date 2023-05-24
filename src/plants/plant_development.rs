use bevy_ecs::prelude::*;
use super::{
    super::{
        environment::params::PhysicalAgeStep,
        branches::branch::*,
        maths::{bounding_box::BoundingBox, bounding_sphere::BoundingSphere},
    },
    plant::*,
};

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

        let new_bounds = BoundingBox::from_spheres(branch_bounds);
        bounds.bounds = new_bounds;
    }
}

// this will calculate all the plant intersections, it will not contain any repeated intersect
pub fn update_plant_intersections(
    mut plants_query: Query<(&PlantBounds, &mut PlantData, Entity), With<PlantTag>>,
) {
    // reset all plant intersection lists
    for (_bounds, mut data, _id) in &mut plants_query {
        data.intersection_list = Vec::new();
    }
    // check all plant intersection options
    let mut combinations = plants_query.iter_combinations_mut();
    while let Some([mut plant_one, plant_two]) = combinations.fetch_next() {
        if plant_one.0.bounds.is_intersecting_box(plant_two.0.bounds) {
            plant_one.1.intersection_list.push(plant_two.2);
        }
    }
}


