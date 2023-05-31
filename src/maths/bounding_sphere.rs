#![allow(dead_code, unused_variables, unused_imports)]
use super::{vector_three::Vector3, colliders::{Collider, RayHitInfo}};
use std::f32::consts::PI;
use std::cmp::{max, min};


#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct BoundingSphere {
    pub centre: Vector3,
    pub radius: f32,
}


impl BoundingSphere {
    pub fn new(centre: impl Into<Vector3>, radius: f32) -> Self {
        BoundingSphere {
            centre: centre.into(),
            radius
        }
    }

    #[allow(non_snake_case)]
    pub fn ZERO() -> Self {
        BoundingSphere {
            centre: Vector3::ZERO(),
            radius: 0.0
        }
    }

    pub fn set_zero(&mut self) {
        self.centre = Vector3 {x: 0.0, y: 0.0, z: 0.0};
        self.radius = 0.0;
    }

    pub fn set_to(&mut self, data: BoundingSphere) {
        self.centre = data.centre;
        self.radius = data.radius;
    }

    pub fn max_dist_from_point(&self, point: Vector3) -> f32 {
        let mut dist_to_centre = (self.centre - point).magnitude();
        dist_to_centre += self.radius;

        dist_to_centre
    }

    pub fn furthest_point_from_point(&self, point: Vector3) -> Vector3 {
        let mut dir_vector = self.centre - point;
        dir_vector.normalise();
        let furthest_point = dir_vector * self.radius + self.centre;

        furthest_point
    }

    pub fn contains_points(&self, points: Vec<Vector3>) -> bool {
        for point in points.iter() {
            let shifted_point_magnitude = (self.centre - *point).magnitude();

            if shifted_point_magnitude > self.radius {
                return false;
            }
        }
        true
    }

    #[allow(unused_assignments)]
    pub fn from_points(points: Vec<Vector3>) -> Self {
        if points.len() == 0 {
            return BoundingSphere::new(Vector3::ZERO(), 0.0)
        }
    
        // get the points with min and max x y and z values
        let mut x_min = points[0];
        let mut y_min = points[0];
        let mut z_min = points[0];
    
        let mut x_max = points[0];
        let mut y_max = points[0];
        let mut z_max = points[0];
    
        for point in points.iter() {
            let x = point.x;
            let y = point.y;
            let z = point.z;
    
            if x < x_min.x {x_min = point.clone()}
            if x > x_max.x {x_max = point.clone()}
    
            if y < y_min.y {y_min = point.clone()}
            if y > y_max.y {y_max = point.clone()}
    
            if z < z_min.z {z_min = point.clone()}
            if z > z_max.z {z_max = point.clone()}
        }
    
        // compute x y and z spans
        let x_span = (x_max - x_min).sqr_magnitude();
        let y_span = (y_max - y_min).sqr_magnitude();
        let z_span = (z_max - z_min).sqr_magnitude();
    
        // set diameter endpoints to largest span
        let mut diameter_one = x_min;
        let mut diameter_two = x_max;
        let mut max_span = x_span;
        if y_span > max_span {
            max_span = y_span;
            diameter_one = y_min;
            diameter_two = y_max;
        }
        if z_span > max_span {
            max_span = z_span;
            diameter_one = z_min;
            diameter_two = z_max;
        }
    
        // calculate the centre and radius of the initial ritter sphere
        let mut ritter_centre = Vector3 {
            x: (diameter_one.x + diameter_two.x) * 0.5,
            y: (diameter_one.y + diameter_two.y) * 0.5,
            z: (diameter_one.z + diameter_two.z) * 0.5,
        };
    
        let mut radius_squared = (diameter_two - ritter_centre).sqr_magnitude();
        let mut ritter_radius = radius_squared.sqrt();
    
        // find the centre of the sphere for the naive method
        let min_box_pt = Vector3 {
            x: x_min.x,
            y: y_min.y,
            z: z_min.z,
        };
        let max_box_pt = Vector3 {
            x: x_max.x,
            y: y_max.y,
            z: z_max.z,
        };
        // let naive_centre = Vector3 {
        //     x: (min_box_pt.x + max_box_pt.x) * 0.5,
        //     y: (min_box_pt.y + max_box_pt.y) * 0.5,
        //     z: (min_box_pt.z + max_box_pt.z) * 0.5,
        // };
        let naive_centre = (max_box_pt + min_box_pt) * 0.5;
    
        // begin second pass to find naive radius and modify ritter sphere
        let mut naive_radius = 0.0;
        for point in points.iter() {
    
            // check if point is furthest from the centre, use furthest point for radius
            let r = (*point - naive_centre).magnitude();
            if r > naive_radius {naive_radius = r}
    
            // make adjustments to ritter sphere to make sure it includes all points
            let old_centre_to_point_squared = (*point - ritter_centre).sqr_magnitude();
            if old_centre_to_point_squared > radius_squared {
    
                let old_centre_to_point = old_centre_to_point_squared.sqrt();
    
                // calculate new radius
                ritter_radius = (ritter_radius + old_centre_to_point) * 0.5;
                radius_squared = ritter_radius * ritter_radius;
                // calculate new ritter centre
                let old_to_new = old_centre_to_point - ritter_radius;
                ritter_centre.x = (ritter_radius * ritter_centre.x + old_to_new * point.x) / old_centre_to_point;
                ritter_centre.y = (ritter_radius * ritter_centre.y + old_to_new * point.y) / old_centre_to_point;
                ritter_centre.z = (ritter_radius * ritter_centre.z + old_to_new * point.z) / old_centre_to_point;
    
            }
    
        }
    
        if ritter_radius < naive_radius {
            BoundingSphere::new(ritter_centre, ritter_radius)
        } else {
            BoundingSphere::new(naive_centre, naive_radius)
        }
    }

    pub fn is_intersecting_sphere(&self, other: BoundingSphere) -> bool {
        let distance_between = (self.centre - other.centre).magnitude();
        let radii_sum = self.radius + other.radius;
        distance_between < radii_sum
    }

    // this function only works if we know that distace <= r1 + r2 but since we'll only call it on bounds we know are intersecting thats fine
    pub fn get_intersection_volume(&self, other: &BoundingSphere) -> f32 {
        let distance = (self.centre - other.centre).magnitude();
        let volume = (PI / (12.0 * distance)) * (self.radius + other.radius - distance).powi(2) * (distance.powi(2) + 2.0 * distance * (self.radius + other.radius) - 3.0 * (self.radius - other.radius).powi(2));
        volume
    }

    pub fn get_volume(&self) -> f32 {
        (4.0 * PI * self.radius * self.radius * self.radius) / 3.0
    }

}

/// solves a quadratic and returns the smallest positive solution
fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<f32>{
    let det = b * b - 4.0 * a * c;
    if det < 0.0 {return None;}
    else if det == 0.0 {
        let s0 = -b / (2.0 * a);
        if s0 < 0.0 {return None;}
        else {return Some(s0);}
    }
    else {
        let (s1, s2) = ((-b + det.sqrt()) / (2.0 * a),(-b - det.sqrt()) / (2.0 * a));
        if s1 > s2 && s2 >= 0.0 {return Some(s2);}
        else if s1 >= 0.0 {return Some(s1);}
        else {return None;}
    }
}

impl Collider for BoundingSphere {
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: Option<f32>,
    ) -> Option<RayHitInfo> {
        let (root_position, direction): (Vector3, Vector3) = (root_position.into(), direction.into());
        let direction = direction.normalised();


        let l = root_position - self.centre;
        if l.magnitude() <= self.radius {
            return Some(RayHitInfo::new(root_position, 0.0));
        }

        let a = direction.dot(direction);
        let b = 2.0 * direction.dot(l);
        let c = l.dot(l) - self.radius * self.radius;
        let dist = solve_quadratic(a, b, c);

        if dist.is_none() {return None;}
        let dist = dist.unwrap();
        if max_distance.is_some() && dist > max_distance.unwrap() {return None;}

        return Some(RayHitInfo::new(root_position + direction * dist, dist));
    }
}


#[cfg(test)]
mod quadratic_tests {
    use super::solve_quadratic;
    #[test]
    fn no_real_solutions_test() {
        assert_eq!(solve_quadratic(1.0, 1.0, 8.0), None)
    }

    #[test]
    fn no_real_positive_solutions_test() {
        assert_eq!(solve_quadratic(1.0, 5.0, 6.0), None)
    }

    #[test]
    fn one_real_positive_solution() {
        assert_eq!(solve_quadratic(1.0,-8.0, 16.0), Some(4.0))
    }

    #[test]
    fn two_real_positive_solution() {
        assert_eq!(solve_quadratic(1.0,-10.0, 16.0), Some(2.0))
    }

    #[test]
    fn zero_is_the_solution() {
        assert_eq!(solve_quadratic(1.0,-10.0, 0.0), Some(0.0))
    }
}

#[cfg(test)]
mod sphere_collider_tests {
    use super::{BoundingSphere, Collider};
    #[test]
    fn contained_point_test() {
        let sphere = BoundingSphere::new([0, 0, 0], 5.0);
        let hit = sphere.check_ray([0.0, 2.5, 0.0], [1, 0, 0], Some(2.0)).unwrap();
        assert_eq!(hit.hit_position, [0.0, 2.5, 0.0].into());
        assert_eq!(hit.hit_distance, 0.0);
    }

    #[test]
    fn intersection_test() {
        let sphere = BoundingSphere::new([0, 0, 0], 2.5);
        let hit = sphere.check_ray([5.0, 0.0, 0.0], [-1, 0, 0], Some(20.0)).unwrap();
        assert_eq!(hit.hit_position, [2.5, 0.0, 0.0].into());
        assert_eq!(hit.hit_distance, 2.5);
    }

    #[test]
    fn too_far_test() {
        let sphere = BoundingSphere::new([0, 0, 0], 2.5);
        let hit = sphere.check_ray([5.0, 0.0, 0.0], [-1, 0, 0], Some(2.0));
        assert!(hit.is_none())
    }

    #[test]
    fn no_intersection_test() {
        let sphere = BoundingSphere::new([0, 0, 0], 2.5);
        let hit = sphere.check_ray([5.0, 0.0, 0.0], [0, 1, 0], Some(25.0));
        assert!(hit.is_none())
    }
}