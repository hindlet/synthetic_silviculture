use bevy_ecs::prelude::*;


pub struct PlantSpeciesData {
    pub seeding_frequency: f32,
    pub seeding_radius: f32,
    pub shadow_tolerance: f32,
}


#[derive(Component)]
pub struct PlantSpeciesRef (pub usize);

#[derive(Resource)]
pub struct PlantSpecies {
    pub species: Vec<PlantSpeciesData>
}

impl PlantSpecies {
    pub fn new(data: Vec<(f32, f32, f32)>) -> Self{
        let mut species = Vec::new();
        for (seeding_frequency, seeding_radius, shadow_tolerance) in data {
            species.push(PlantSpeciesData {
                seeding_frequency, seeding_radius, shadow_tolerance
            })
        }
        PlantSpecies{species}
    }
}




