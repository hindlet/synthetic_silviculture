use bevy_ecs::prelude::*;
use super::{
    super::{
        branches::{branch::*, branch_sorting::*},
        maths::{bounding_box::BoundingBox, bounding_sphere::BoundingSphere},
    },
    plant::*,
};

/// steps the ages of all plants by the phyical age step
/// also adjusts max vigor where appropriate
pub fn step_plant_age(
    plants: &mut Vec<Plant>,
    timestep: f32,
    death_rate: f32,
) {

    for plant in plants.iter_mut() {
        plant.age += timestep;

        if plant.age > plant.growth_factors.max_age {
            plant.growth_factors.max_vigor = (plant.growth_factors.max_vigor - death_rate).max(0.0);
        }
    }

}

/// takes data from the branches and distributes it, we do a tip to base pass and sum light exposure at branching points
/// after this we use a helper function to distribute growth vigor up the plant
/// this means that branches closer to the root have a higher growth vigor than those further away
pub fn calculate_growth_vigor(
    plants: &mut Vec<Plant>,
) {

    for i in 0..plants.len() {

        let max_vigor = plants[i].growth_factors.max_vigor;
        let min_vigor = plants[i].growth_factors.min_vigor;
        let apical = plants[i].growth_factors.apical_control;

        let mut layers = get_mut_branch_layers(&mut plants[i].root);

        // distr light down
        for i in (1..layers.len()).rev() {

            for branch in layers[i - 1] {
                branch.growth_data.light_exposure = 0.0;
            }

            for branch in layers[i] {
                layers[i - 1][branch.parent_index].growth_data.light_exposure += branch.growth_data.light_exposure;
            }

        }

        // convert light to vigor
        layers[0][0].growth_data.growth_vigor = layers[0][0].growth_data.light_exposure;

        // distr vigor up
        for i in 0..layers.len() - 1 {

            for branch in layers[i] {

                if let Some(child_one) = branch.children.0 {

                    if let Some(child_two) = branch.children.1 {

                        let vigors = calculate_child_vigor(branch.growth_data.growth_vigor, child_one.growth_data.light_exposure, child_two.growth_data.light_exposure, apical);
                        child_one.growth_data.growth_vigor = vigors.0;
                        child_two.growth_data.growth_vigor = vigors.1;

                    }
                    else {
                        child_one.growth_data.growth_vigor = branch.growth_data.growth_vigor;
                    }
                }
            }
        }


    }

}


fn calculate_child_vigor(
    parent_vigor: f32,
    child_one_light: f32,
    child_two_light: f32,
    apical: f32,
) -> (f32, f32){

    // there is a main branch and a lateral branch, the main branch is the branch with the most light exposure
    // check which branch is main and use that to calculate using Vm = Vp * (apical * Qm) / (apical * Qm + 1-apical * Ql)
    // if they are the same then just split the vigor evenly

    if child_one_light == child_two_light {
        return (parent_vigor / 2.0, parent_vigor / 2.0);
    }
    else if child_one_light > child_two_light {
        let greater_vigor = parent_vigor * apical * child_one_light / (apical * child_one_light + (1.0 - apical) * child_two_light);
        (greater_vigor, 1.0 - greater_vigor)
    }
    else {
        let greater_vigor = parent_vigor * apical * child_two_light / (apical * child_two_light + (1.0 - apical) * child_one_light);
        (1.0 - greater_vigor, greater_vigor)
    }


}




// // must be called after updating all the branch bounds
// pub fn update_plant_bounds(
//     branch_bounds_query: Query<&BranchBounds, With<BranchTag>>,
//     branch_connections_query: Query<&BranchConnectionData, With<BranchTag>>,
//     mut plants_query: Query<(&mut PlantBounds, &PlantData), With<PlantTag>>
// ) {
//     for (mut bounds, plant_data) in &mut plants_query {
//         if plant_data.root_node.is_none() {continue;}

//         let mut branch_bounds: Vec<BoundingSphere> = Vec::new();

        
//         for id in get_branches_base_to_tip(&branch_connections_query, plant_data.root_node.unwrap()) {
//             if let Ok(bounds) = branch_bounds_query.get(id) {
//                 branch_bounds.push(bounds.bounds.clone());
//             }
//         }

//         let new_bounds = BoundingBox::from_spheres(branch_bounds);
//         bounds.bounds = new_bounds;
//     }
// }

// // this will calculate all the plant intersections, it will not contain any repeated intersections
// pub fn update_plant_intersections(
//     mut plants_query: Query<(&PlantBounds, &mut PlantData, Entity), With<PlantTag>>,
// ) {
//     // reset all plant intersection lists
//     for (_bounds, mut data, _id) in &mut plants_query {
//         data.intersection_list = Vec::new();
//     }
//     // check all plant intersection options
//     let mut combinations = plants_query.iter_combinations_mut();
//     while let Some([mut plant_one, plant_two]) = combinations.fetch_next() {
//         if plant_one.0.bounds.is_intersecting_box(plant_two.0.bounds) {
//             plant_one.1.intersection_list.push(plant_two.2);
//         }
//     }
// }


