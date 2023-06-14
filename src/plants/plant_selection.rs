use rand::Rng;
use super::{
    plant::{PlantGrowthControlFactors, PlantPlasticityParameters},
    super::maths::{normal_cmd, normal_probabilty_density},
};


/// A sampler used to select which plants are initally spawned into an environment
/// 
/// - species: A vec of all the different plant species data, their index corresponds to a set of params
/// - species_params: A vec of the conditions needed for different species to be created, (ideal_temp, temp_standard_deviation, ideal_temp_prob_density ideal_moisture, moisture_standard_deviation, ideal_moisture_prob_density)
pub struct PlantSpeciesSampler {
    species: Vec<(PlantGrowthControlFactors, PlantPlasticityParameters)>,
    species_params: Vec<(f32, f32, f32, f32)>,
}

impl PlantSpeciesSampler {

    /// creates a new plant_species sampler from the given plant species and parameters
    pub fn new(init_species: Vec<((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))>) -> Self {
        let mut species_params = Vec::new();
        let mut plants = Vec::new();
        for species in init_species {
            let mut growth_factors = species.0.0;
            growth_factors.max_vigor = growth_factors.species_max_vigor;
            species_params.push((species.1.0, species.1.1, species.1.2, species.1.3,));
            plants.push((growth_factors, species.0.1));
        }
        PlantSpeciesSampler {
            species: plants, species_params
        }
    }

    /// adds new plant species to the sampler
    pub fn add_species(&mut self, new_species: Vec<((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))>) {
        let mut species_params = Vec::new();
        let mut plants = Vec::new();
        for species in new_species {
            species_params.push((species.1.0, species.1.1, species.1.2, species.1.3));
            plants.push(species.0);
        }
        self.species.append(&mut plants);
        self.species_params.append(&mut species_params)
    }

    /// replaces all the species in the sampler with the given options
    pub fn replace_all(&mut self, new_species: Vec<((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))>) {
        let mut species_params = Vec::new();
        let mut plants = Vec::new();
        for species in new_species {
            species_params.push((species.1.0, species.1.1, species.1.2, species.1.3));
            plants.push(species.0);
        }
        self.species = plants;
        self.species_params = species_params;
    }

    /// removes a species from the sampler at a given index
    pub fn remove(&mut self, index: usize) {
        self.species.remove(index);
        self.species.remove(index);
    }

    /// replaces a species in the sampler at a given index with the provided species
    pub fn replace(&mut self, index: usize, new: ((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))) {
        self.remove(index);
        self.species.insert(index, new.0);
        let params = (new.1.0, new.1.1, new.1.2, new.1.3);
        self.species_params.insert(index, params);
    }



    /// returns a set of plant control factors based on temperature and moisture
    /// 
    /// - Plants lying more than 5 standard deviations away from either parameter are removed from the chances, probability at that point is close to 0
    /// - All remaining plants are chosen from with probabilty weights generated using normal distribution
    /// - If no plants can be grown, returns None
    /// - Returns the chosen plant and it's
    pub fn get_plant(&self, temp: f32, moist: f32) -> Option<((PlantGrowthControlFactors, PlantPlasticityParameters), f32)>{

        let mut choices: Vec<(usize, f32)> = Vec::new();
        let mut total_prob = 0.0;

        for i in 0..self.species_params.len() {
            let (ideal_temp, std_dev_temp, ideal_moist, std_dev_moist) = self.species_params[i];

            // disregard any plants more than 5 standard deviations away in any direction
            if (temp - ideal_temp).abs() > 5.0 * std_dev_temp || (moist - ideal_moist).abs() > 5.0 * std_dev_moist {continue;}

            let temp_prob = normal_probabilty_density(temp, ideal_temp, std_dev_temp) / normal_probabilty_density(ideal_temp, ideal_temp, std_dev_temp);
            let moist_prob = normal_probabilty_density(moist, ideal_moist, std_dev_moist) / normal_probabilty_density(ideal_moist, ideal_moist, std_dev_moist);

            total_prob += temp_prob * moist_prob;
            choices.push((i, temp_prob * moist_prob));
        }
        let position = rand::thread_rng().gen_range(0.0..=total_prob);

        let mut total_prob = 0.0;
        for choice in choices.iter() {
            total_prob += choice.1;
            if position <=  total_prob{
                return Some((self.species[choice.0].clone(), choice.1));
            }
        }
        None
    }
}
