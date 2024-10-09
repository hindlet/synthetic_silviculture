use crate::{branch::Branch, plant::Plant, plant_species::PlantSpeciesList};



pub struct Environment<'branch_lifetime, 'node_lifetime> {
    pub plants: Vec<Plant<'branch_lifetime, 'node_lifetime>>,
    pub plant_species: PlantSpeciesList
}

impl<'branch_lifetime, 'node_lifetime> Environment<'branch_lifetime, 'node_lifetime> {



    /// completes one step of the plant simulation
    pub fn step_simulation(&mut self) {

        ////// pre-running steps
        plant_death(&mut self.plants, &self.plant_species);


        // generate a list of mutable references to all branches
        // this list will mean that branches that are children will occur in the list after their parents
        let mut branch_list = Vec::new();

        let branches: Vec<Vec<&mut Branch>> = self.plants.iter_mut().map(|p| p.branches.iter_mut().map(|(_, b)| b).collect::<Vec<&mut Branch>>()).collect::<Vec<Vec<&mut Branch>>>();
        for plant_branches in branches {
            branch_list.extend(plant_branches);
        }

        // Step 1: Recalculate Branch Bounding Volumes and reset intersection volume
        recalculate_branch_bounds(&mut branch_list);
        // Step 2: Calculate Branch Light Exposure and Sum it for the plant
        calculate_and_sum_light_exposure(&mut branch_list);
        // Step 3: Calculate and distribute growth vigor
        calculate_and_distr_growth_vigor(&mut self.plants, &self.plant_species);



    }

}

fn plant_death(
    plants: &mut Vec<Plant>,
    species: &PlantSpeciesList
) {
    for plant in plants.iter_mut() {
        let species_ref = species.species.get(&plant.species_id).unwrap();
        if plant.age < species_ref.max_healthy_age {continue;} // if the plant is young skip
        plant.max_vigor = (plant.max_vigor - species_ref.death_rate).max(0.0); // otherwise reduce the plants max vigor by the death rate
    }
}


fn recalculate_branch_bounds(
    branches: &mut Vec<&mut Branch<'_, '_>>
) {

    // iterate over all branches
    for branch in branches.iter_mut() {
        branch.calculate_bounding_volume();
        branch.intersection_volume = 0.0;
    }
}



fn calculate_and_sum_light_exposure(
    branches: &mut Vec<&mut Branch<'_, '_>>
) {    

    let mut volume_sum: f32;

    // iterate through each pair of branches
    for i in 0..branches.len() {
        // reset the volume sum and get the bounding sphere of the first branch of the pair
        volume_sum = 0.0;
        let branch_one_bounds = branches[i].bounding_volume;
        // iterate over all the other branches to get pairs and calculate the intersection volume
        for j in (i+1)..branches.len() {
            let branch_two = &mut branches[j];
            let volume = branch_one_bounds.get_intersection_volume(&branch_two.bounding_volume);
            branch_two.intersection_volume += volume;
            volume_sum += volume;
        }
        branches[i].intersection_volume = volume_sum;
    }


    
    // iterate down the branches to distriute light exposure
    for i in (0..branches.len()).rev() {
        let branch = &mut branches[i];
        match (&branch.child_one, &branch.child_two) {
            (Some(child_one), Some(child_two)) => {
                branch.light_sum_at_point = child_one.light_sum_at_point + child_two.light_sum_at_point;
            },
            (Some(child_one), None) => {
                branch.light_sum_at_point = child_one.light_sum_at_point;
            }
            (None, Some(child_two)) => {
                branch.light_sum_at_point = child_two.light_sum_at_point;
            }
            (None, None) => {
                branch.light_sum_at_point = (-branch.intersection_volume).exp()
            }
        }
    }
}

fn calculate_and_distr_growth_vigor(
    plants: &mut Vec<Plant>,
    species: &PlantSpeciesList
) {
    for plant in plants.iter_mut() {
        let species = species.species.get(&plant.species_id).unwrap();

        for (branch_id, branch) in plant.branches.iter_mut() {

            // set own growth vigor, root branches convert light to vigor
            if *branch_id == 0 {
                branch.growth_vigor = branch.light_sum_at_point.min(plant.max_vigor);
            } else {
                branch.growth_vigor = if branch.first_child {
                    branch.parent.child_vigor.0
                } else {
                    branch.parent.child_vigor.1
                }
            }

            match (&branch.child_one, &branch.child_two) {
                (Some(child_one), Some(child_two)) => {
                    if child_one.light_sum_at_point > child_two.light_sum_at_point {
                        let main_child_vigor = branch.growth_vigor * (species.apical_control * child_one.light_sum_at_point) / (species.apical_control * child_one.light_sum_at_point + (1.0 - species.apical_control) * child_two.light_sum_at_point);
                        branch.child_vigor = (main_child_vigor, branch.growth_vigor - main_child_vigor)
                    } else {
                        let main_child_vigor = branch.growth_vigor * (species.apical_control * child_two.light_sum_at_point) / (species.apical_control * child_two.light_sum_at_point + (1.0 - species.apical_control) * child_one.light_sum_at_point);
                        branch.child_vigor = (branch.growth_vigor - main_child_vigor, main_child_vigor)
                    }
                },
                (Some(_), None) => {
                    branch.child_vigor = (branch.growth_vigor, 0.0)
                }
                (None, Some(_)) => {
                    branch.child_vigor = (0.0, branch.growth_vigor);
                }
                (None, None) => {}
            }
        }

    }
}


