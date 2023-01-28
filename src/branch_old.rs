#![allow(dead_code, unused_variables, unused_imports, unused_assignments)]

#[cfg(test)]
mod tests {
    use super::{Branch, BranchNode};

    // checks if all branch nodes are in the bounding sphere
    fn check_all_nodes_in(branch: &Branch) -> bool{
        let centre = branch.bounding_sphere.0;
        let radius_squared = branch.bounding_sphere.1 * branch.bounding_sphere.1;

        for node in branch.nodes.iter() {
            let position = node.position;

            let equation_substition = (position[0] - centre[0])*(position[0] - centre[0]) + (position[1] - centre[1])*(position[1] - centre[1]) + (position[2] - centre[2])*(position[2] - centre[2]);
            
            if equation_substition > radius_squared {
                return false;
            }
        }

        true
    }

    #[test]
    fn bounds_test_zero_nodes() {
        let mut test_branch_one = Branch {
            nodes: vec![],
            edges: vec![],
            bounding_sphere: ([5.0, 5.0, 6.0], 12.0),
            ..Default::default()
        };
        test_branch_one.update_bounds();
        assert_eq!(test_branch_one.bounding_sphere, ([0.0, 0.0, 0.0], 0.0));
    }

    #[test]
    fn bounds_test_defined_sphere() {
        let defined_sphere_branch = Branch {
            nodes: vec![
                BranchNode {position: [0.0, 0.0, 0.0]},
                BranchNode {position: [0.0, 2.5, 0.0]},
                BranchNode {position: [0.0, 1.5, 1.5]},
                ],
            edges: vec![],
            bounding_sphere: ([0.0, 0.0, 0.0], 2.5),
            ..Default::default()
        };
        assert_eq!(check_all_nodes_in(&defined_sphere_branch), true);
    }

    #[test]
    fn bounds_test_calculated_bounds() {
        let mut calculate_sphere_branch = Branch {
            nodes: vec![
                BranchNode {position: [0.0, 0.0, 0.0]},
                BranchNode {position: [0.0, 2.5, 0.0]},
                BranchNode {position: [0.0, 1.5, 1.5]},
                ],
            edges: vec![],
            bounding_sphere: ([0.0, 0.0, 0.0], 0.0),
            ..Default::default()
        };
        calculate_sphere_branch.update_bounds();
        assert_eq!(check_all_nodes_in(&calculate_sphere_branch), true);
    }
}


#[derive(Clone)]
struct BranchNode {
    position: [f32; 3], // this will be a world position
}

#[derive(Clone)]
pub struct Branch {
    // permentant data
    nodes: Vec<BranchNode>,
    edges: Vec<u32>,
    age: f32,
    thickening_factor: f32,
    
    // used in calculations every update
    bounding_sphere: ([f32; 3], f32), // centre, radius
    intersection_indexes: Vec<(usize, usize)>, // tree index, branch index
    intersection_volume: f32,
    light_exposure: f32,
    vigor: f32,
}

impl Default for Branch {
    fn default() -> Branch {
        Branch {
            nodes: vec![],
            edges: vec![],
            
            age: 0.0,
            thickening_factor: 0.0,

            bounding_sphere: ([0.0, 0.0, 0.0], 0.0),
            intersection_indexes: vec![],
            intersection_volume: 0.0,
            light_exposure: 0.0,
            vigor: 0.0,
        }
    }
}


impl Branch {
    fn update_bounds(&mut self) { // generates the bounding sphere from the node positions of the branch
        if self.nodes.len() == 0 {
            self.bounding_sphere = ([0.0, 0.0, 0.0], 0.0);
            return;
        }

        // these should be vec3
        let mut xmin = self.nodes[0].position;
        let mut ymin = self.nodes[0].position;
        let mut zmin = self.nodes[0].position;

        let mut xmax = self.nodes[0].position;
        let mut ymax = self.nodes[0].position;
        let mut zmax = self.nodes[0].position;

        for node in self.nodes.iter() {
            let x = node.position[0];
            let y = node.position[1];
            let z = node.position[2];

            // check maxes and mins
            if x < xmin[0] {xmin = node.position}
            if x > xmax[0] {xmax = node.position}

            if y < ymin[1] {ymin = node.position}
            if y > ymax[1] {ymax = node.position}
            
            if z < zmin[2] {zmin = node.position}
            if z > zmax[2] {zmax = node.position}
        }

        // compute x, y and z spans
        let xspan = {
            let balls = [xmax[0] - xmin[0], xmax[1] - xmin[1], xmax[2] - xmax[2]];
            balls[0] * balls[0] + balls[1] + balls[1] + balls[2] * balls[2]
        };
        let yspan = {
            let balls = [ymax[0] - ymin[0], ymax[1] - ymin[1], ymax[2] - ymax[2]];
            balls[0] * balls[0] + balls[1] + balls[1] + balls[2] * balls[2]
        };
        let zspan = {
            let balls = [zmax[0] - zmin[0], zmax[1] - zmin[1], zmax[2] - zmax[2]];
            balls[0] * balls[0] + balls[1] + balls[1] + balls[2] * balls[2]
        };

        // set diameter endpoints to the largest span
        let mut diameter1 = xmin;
        let mut diameter2 = xmax;
        let mut maxspan = xspan;
        if yspan > maxspan {
            maxspan = yspan;
            diameter1 = ymin;
            diameter2 = ymax;
        }
        if zspan > maxspan {
            
            maxspan = zspan;
            diameter1 = zmin;
            diameter2 = zmax;
        }

        // calculate the centre of the initial sphere found by ritters algorithm
        let mut ritter_centre = [0.0, 0.0, 0.0];
        ritter_centre[0] = (diameter1[0] + diameter2[0]) * 0.5;
        ritter_centre[1] = (diameter1[1] + diameter2[1]) * 0.5;
        ritter_centre[2] = (diameter1[2] + diameter2[2]) * 0.5;

        // calculate the radius of the initial sphere
        let mut radius_squared = {
            let balls = [diameter2[0] - ritter_centre[0], diameter2[1] - ritter_centre[1], diameter2[2] - ritter_centre[2]];
            balls[0] * balls[0] + balls[1] + balls[1] + balls[2] * balls[2]
        };
        let mut ritter_radius = radius_squared.sqrt();


        // find the centre of the sphere found using the naive method
        let min_box_pt = [xmin[0], ymin[1], zmin[2]];
        let max_box_pt = [xmax[0], ymax[1], zmax[2]];
        let naive_centre = [
            (min_box_pt[0] + max_box_pt[0]) * 0.5,
            (min_box_pt[1] + max_box_pt[1]) * 0.5,
            (min_box_pt[2] + max_box_pt[2]) * 0.5,
        ];

        // begin second pass to find naive radius and modify ritter
        let mut naive_radius = 0.0;
        for node in self.nodes.iter() {
            let position = node.position;

            // find the furthest point from the centre to calculate the radius
            let r = {
                let balls = [position[0] - naive_centre[0], position[1] - naive_centre[1], position[2] - naive_centre[2]];
                balls[0] * balls[0] + balls[1] + balls[1] + balls[2] * balls[2]
            };
            if r > naive_radius {naive_radius = r};
            

            // make adjustments to ritter sphere to include all points
            let old_centre_to_point_squared = {
                let balls = [position[0] - ritter_centre[0], position[1] - ritter_centre[1], position[2] - ritter_centre[2]];
                balls[0] * balls[0] + balls[1] + balls[1] + balls[2] * balls[2]
            };

            if old_centre_to_point_squared > radius_squared {
                let old_centre_to_point = old_centre_to_point_squared.sqrt();

                // calculate new radius to include the point that lies outisde
                ritter_radius = (ritter_radius + old_centre_to_point) * 0.5;
                radius_squared = ritter_radius * ritter_radius;
                // calculate new centre of the ritter sphere
                let old_to_new = old_centre_to_point - ritter_radius;
                ritter_centre[0] = (ritter_radius * ritter_centre[0] + old_to_new * position[0]) / old_centre_to_point;
                ritter_centre[1] = (ritter_radius * ritter_centre[1] + old_to_new * position[1]) / old_centre_to_point;
                ritter_centre[2] = (ritter_radius * ritter_centre[2] + old_to_new * position[2]) / old_centre_to_point;
            }
        }
        // choose the smaller of the two spheres
        if ritter_radius < naive_radius {
            self.bounding_sphere = (ritter_centre, ritter_radius);
        } else {
            self.bounding_sphere = (naive_centre, naive_radius);
        }
    }

    fn update_intersect_volumes(&mut self) {
        for index in self.intersection_indexes.iter() {

        }
    }


    fn get_bounds_ref(&self) -> &([f32; 3], f32) {
        &self.bounding_sphere
    }


}

fn are_branches_intersecting(branch_one: &Branch, branch_two: &Branch) -> bool {
    // get the distance between the two bounding sphere centres, if it is less than the sum of radii then they are intersecting
    let sphere_one = branch_one.bounding_sphere;
    let sphere_two = branch_two.bounding_sphere;

    let square_distance = {
        let distance_vector = [
            sphere_one.0[0] - sphere_two.0[0],
            sphere_one.0[1] - sphere_two.0[1],
            sphere_one.0[2] - sphere_two.0[2],
        ];
        distance_vector[0] * distance_vector[0] + distance_vector[1] * distance_vector[1] + distance_vector[2] * distance_vector[2]
    };

    let intersecting = square_distance < ((sphere_one.1 + sphere_two.1) * (sphere_one.1 + sphere_two.1));
    intersecting
}



pub struct BranchPrototypes {
    prototypes: Vec<Vec<Branch>>,
}

impl BranchPrototypes {
    fn get_prototype(&self, apical_control: usize, determinancy: usize) -> Branch{
        self.prototypes[apical_control][determinancy].clone()
    }
}