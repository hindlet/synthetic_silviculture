use std::collections::HashMap;




pub struct PlantSpecies {
    pub apical_control: f32,
    pub healthy_maxiumum_vigor: f32,
    pub shedding_threshold: f32,

    pub max_healthy_age: f32,
    pub death_rate: f32,
}


pub struct PlantSpeciesList {
    pub species: HashMap<usize, PlantSpecies>
}