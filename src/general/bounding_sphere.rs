
use bevy_ecs::prelude::*;
use crate::general::vector_three::Vector3;


#[derive(Default, Component, Debug, PartialEq, Clone, Copy)]
pub struct BoundingSphere {
    pub centre: Vector3,
    pub radius: f32,
}

use std::cmp::{max, min};
use std::f32::consts::PI;

impl BoundingSphere {
    pub fn new() -> Self {
        BoundingSphere {
            centre: Vector3::new(),
            radius: 0.0
        }
    }

    pub fn set_zero(&mut self) {
        self.centre = Vector3 {x: 0.0, y: 0.0, z: 0.0};
        self.radius = 0.0;
    }

    pub fn set_to(&mut self, data: &BoundingSphere) {
        self.centre = data.centre;
        self.radius = data.radius;
    }

    pub fn max_dist_from_point(&self, point: Vector3) -> f32 {
        let mut dist_to_centre = self.centre.subtract_to_new(&point).get_magnitude();
        dist_to_centre += self.radius;

        dist_to_centre
    }

    pub fn furthest_point_from_point(&self, point: &Vector3) -> Vector3 {
        let mut dir_vector = self.centre.subtract_to_new(point);
        dir_vector.normalise();
        let furthest_point = dir_vector.multiply_to_new(self.radius).add_to_new(&self.centre);

        furthest_point
    }

    pub fn contains_points(&self, points: &Vec<Vector3>) -> bool {
        for point in points.iter() {
            let shifted_point_magnitude = self.centre.subtract_to_new(point).get_magnitude();

            if shifted_point_magnitude > self.radius {
                return false;
            }
        }
        true
    }

    #[allow(unused_assignments)]
    pub fn from_points(points: &Vec<Vector3>) -> Self {
        if points.len() == 0 {
            return BoundingSphere {
                centre: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                radius: 0.0
            };
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
        let x_span = x_max.subtract_to_new(&x_min).get_sqr_magnitude();
        let y_span = y_max.subtract_to_new(&y_min).get_sqr_magnitude();
        let z_span = z_max.subtract_to_new(&z_min).get_sqr_magnitude();
    
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
    
        let mut radius_squared = diameter_two.subtract_to_new(&ritter_centre).get_sqr_magnitude();
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
        let naive_centre = Vector3 {
            x: (min_box_pt.x + max_box_pt.x) * 0.5,
            y: (min_box_pt.y + max_box_pt.y) * 0.5,
            z: (min_box_pt.z + max_box_pt.z) * 0.5,
        };
    
        // begin second pass to find naive radius and modify ritter sphere
        let mut naive_radius = 0.0;
        for point in points.iter() {
    
            // check if point is furthest from the centre, use furthest point for radius
            let r = point.clone().subtract_to_new(&naive_centre).get_magnitude();
            if r > naive_radius {naive_radius = r}
    
            // make adjustments to ritter sphere to make sure it includes all points
            let old_centre_to_point_squared = point.clone().subtract_to_new(&ritter_centre).get_sqr_magnitude();
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
            BoundingSphere {
                centre: ritter_centre,
                radius: ritter_radius,
            }
        } else {
            BoundingSphere {
                centre: naive_centre,
                radius: naive_radius,
            }
        }
    }

    pub fn is_intersecting_sphere(&self, other: &BoundingSphere) -> bool {
        let distance_between = self.centre.subtract_to_new(&other.centre).get_magnitude();
        let radii_sum = self.radius + other.radius;
        distance_between < radii_sum
    }

    // this function only works if we know that distace <= r1 + r2 but since we'll only call it on bounds we know are intersecting thats fine
    pub fn get_intersection_volume(&self, other: &BoundingSphere) -> f32 {
        let distance = self.centre.subtract_to_new(&other.centre).get_magnitude();
        let volume = (PI / (12.0 * distance)) * (self.radius + other.radius - distance).powi(2) * (distance.powi(2) + 2.0 * distance * (self.radius + other.radius) - 3.0 * (self.radius - other.radius).powi(2));
        volume
    }

}