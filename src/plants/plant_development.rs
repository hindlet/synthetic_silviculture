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
        plant_data.age += timestep.step * plant_growth_data.growth_rate;
        if plant_data.age > plant_growth_data.max_age {
            plant_growth_data.max_vigor = 0.0_f32.max(plant_growth_data.max_vigor - descrease_rate.v_max_decrease);
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


