use crate::{branch::Branch, plant::Plant};



pub struct Environment<'branch_lifetime, 'node_lifetime> {
    pub plants: Vec<Plant<'branch_lifetime, 'node_lifetime>>
}

impl<'branch_lifetime, 'node_lifetime> Environment<'branch_lifetime, 'node_lifetime> {



    /// completes one step of the plant simulation
    pub fn step_simulation(&mut self) {

        self.step_one();
        self.step_two();
        //
        // 
        //






    }

    /// Step 1: Recalculate Branch Bounding Volumes and reset intersection volume
    fn step_one(&mut self) {
        for plant in self.plants.iter_mut() {
            for (_, branch) in plant.branches.iter_mut() {
                branch.calculate_bounding_volume();
                branch.intersection_volume = 0.0;
            }
        }
    }


    /// Step 2: Calculate Branch Light Exposure and Sum it for the plant
    fn step_two(&mut self) {

        // collect a list of lists of all the branches in each plant
        let mut branch_list = Vec::new();

        let branches: Vec<Vec<&mut Branch>> = self.plants.iter_mut().map(|p| p.branches.iter_mut().map(|(_, b)| b).collect::<Vec<&mut Branch>>()).collect::<Vec<Vec<&mut Branch>>>();
        for plant_branches in branches {
            branch_list.extend(plant_branches);
        }

        let mut volume_sum: f32;

        // iterate through each pair of branches
        for i in 0..branch_list.len() {
            // reset the volume sum and get the bounding sphere of the first branch of the pair
            volume_sum = 0.0;
            let branch_one_bounds = branch_list[i].bounding_volume;
            // iterate over all the other branches to get pairs and calculate the intersection volume
            for j in (i+1)..branch_list.len() {
                let branch_two = &mut branch_list[j];
                let volume = branch_one_bounds.get_intersection_volume(&branch_two.bounding_volume);
                branch_two.intersection_volume += volume;
                volume_sum += volume;
            }
            branch_list[i].intersection_volume = volume_sum;
        }


        
        // iterate down the branches to distriute light exposure
        for i in (0..branch_list.len()).rev() {
            let branch = &mut branch_list[i];
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


    /// Step 3: Calculate and distribute growth vigor
    fn step_three(&mut self) {

    }

}