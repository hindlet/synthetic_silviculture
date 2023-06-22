use bevy_ecs::prelude::*;
use std::f32::consts::PI;
use rand_distr::{Normal, Distribution};
use rand::{thread_rng, Rng};
use super::{
    super::{
        environment::{
            params::*,
            terrain::*,
        },
        branches::{branch::*, branch_prototypes::BranchPrototypesSampler},
        maths::{bounding_box::BoundingBox, bounding_sphere::BoundingSphere, colliders::Collider},
    },
    plant::*,
    plant_selection::*,
};
#[cfg(feature = "vulkan_graphics")]
use super::super::graphics::branch_mesh_gen::*;

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


pub fn seed_plants(
    mut plants_query: Query<(&PlantData, &mut PlantGrowthControlFactors, &mut PlantPlasticityParameters, Entity), With<PlantTag>>,
    plant_sampler: Res<PlantSpeciesSampler>,
    environment: Res<MoistureAndTemp>,
    branch_query: Query<&BranchGrowthData, With<BranchTag>>,
    branch_sampler: Res<BranchPrototypesSampler>,
    terrain_query: Query<&TerrainCollider, With<TerrainTag>>,
    timestep: Res<PhysicalAgeStep>,
    #[cfg(feature = "vulkan_graphics")]
    mut queue: Query<&mut MeshUpdateQueue>,
    mut commands: Commands,
) {
    let terrain = terrain_query.single();
    #[cfg(feature = "vulkan_graphics")]
    let mut queue = queue.single_mut();

    for mut plant in plants_query.iter_mut() {
        // check if plant is now flowering
        if !plant.2.is_seeding {
            if plant.0.root_node.is_none() {continue;}
            let root_vigor = if let Ok(root_branch) = branch_query.get(plant.0.root_node.unwrap()) {
                root_branch.growth_vigor
            } else {panic!("Failed to get root branch in fn seed_plants")};
            let effective_flowering_age = plant.2.flowering_age * plant.1.species_max_vigor / root_vigor;
            if plant.0.age >= effective_flowering_age {plant.2.is_seeding = true;}
        }

        // if flowering, add to time since last flower
        if plant.2.is_seeding {
            plant.2.time_since_seeding += timestep.step;
        }

        // check to seed

        while plant.2.time_since_seeding >= plant.2.seeding_interval {
            let distance_from_centre = (Normal::new(plant.2.seeding_radius, plant.2.seeding_std_dev).unwrap().sample(&mut rand::thread_rng()) - plant.2.seeding_radius).abs();
            let angle_from_centre = rand::thread_rng().gen_range(0.0..(PI * 2.0)); // 0 is along the +x axis
            let (ray_x, ray_z) = (plant.0.position.x + angle_from_centre.cos() * distance_from_centre, plant.0.position.z + angle_from_centre.sin() * distance_from_centre);
            if let Some(ray_hit) = terrain.collider.check_ray([ray_x, terrain.max_height, ray_z], [0, -1, 0], None) {
                let child_factors = (plant.1.copy_for_new_plant(), plant.2.copy_for_new_plant());
                let climate_adapt = plant_sampler.calculate_child_climate_adapt(&child_factors, environment.moisture, environment.temp_at_zero + ray_hit.hit_position.y * environment.temp_fall_off);
                let ids = spawn_plant(ray_hit.hit_position, ray_hit.hit_normal, child_factors.0, child_factors.1, climate_adapt, branch_sampler.as_ref(), &mut commands);
                #[cfg(feature = "vulkan_graphics")]
                queue.ids.push_back(ids.1);
            }
            plant.2.time_since_seeding -= plant.2.seeding_interval;
        }


        
        
    }


}

