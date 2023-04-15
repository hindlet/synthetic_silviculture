#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::maths::vector_three::*;
use crate::maths::bounding_sphere::BoundingSphere;


#[derive(Default, Component, Debug, PartialEq, Clone, Copy)]
pub struct BoundingBox {
    pub least_corner: Vector3, // least x y and z
    pub width: f32, // x
    pub height: f32, // y
    pub depth: f32, // z
}


impl BoundingBox {
    pub fn new(least_corner: Vector3, width: f32, height: f32, depth: f32) -> Self {
        BoundingBox {
            least_corner,
            width,
            height,
            depth,
        }
    }

    #[allow(non_snake_case)]
    pub fn ZERO() -> Self {
        Self {
            least_corner: Vector3::ZERO(),
            width: 0.0,
            height: 0.0,
            depth: 0.0,
        }
    }


    pub fn contains_point(&self, point: &Vector3) -> bool {
        if (point.x < self.least_corner.x) || (point.x > (self.least_corner.x + self.width)) {
            return false
        }
        if (point.y < self.least_corner.y) || (point.y > (self.least_corner.y + self.height)) {
            return false
        }
        if (point.z < self.least_corner.z) || (point.z > (self.least_corner.z + self.depth)) {
            return false
        }
        true
    }

    pub fn contains_points(&self, points: &Vec<Vector3>) -> bool {
        for point in points {
            if !self.contains_point(point) {return false}
        }
        true
    }

    pub fn contains_sphere(&self, sphere: &BoundingSphere) -> bool{
        if !self.contains_point(&sphere.centre) {return false;}
        let sphere_max = sphere.centre - Vector3::new(sphere.radius, sphere.radius, sphere.radius);
        assert!(sphere_max < self.least_corner + Vector3::new(self.width, self.height, self.depth));

        if (sphere.centre.x - sphere.radius) < self.least_corner.x {return false;}
        if (sphere.centre.x + sphere.radius) > (self.least_corner.x + self.width) {return false;}

        if (sphere.centre.y - sphere.radius) < self.least_corner.y {return false;}
        if (sphere.centre.y + sphere.radius) > (self.least_corner.y + self.height) {return false;}

        if (sphere.centre.z - sphere.radius) < self.least_corner.z {return false;}
        if (sphere.centre.z + sphere.radius) > (self.least_corner.z + self.depth) {return false;}

        true
    }

    pub fn contains_spheres(&self, spheres: &Vec<BoundingSphere>) -> bool {
        for sphere in spheres.iter() {
            if !self.contains_sphere(&sphere) {return  false}
        }
        true
    }

    pub fn from_spheres(spheres: &Vec<BoundingSphere>) -> BoundingBox {
        if spheres.len() == 0 {
            return BoundingBox::ZERO();
        }


        let mut x_min = spheres[0].centre.x - spheres[0].radius;
        let mut x_max = spheres[0].centre.x + spheres[0].radius;

        let mut y_min = spheres[0].centre.y - spheres[0].radius;
        let mut y_max = spheres[0].centre.y + spheres[0].radius;

        let mut z_min = spheres[0].centre.z - spheres[0].radius;
        let mut z_max = spheres[0].centre.z + spheres[0].radius;

        for sphere in spheres.iter() {
            let sphere_x_min = sphere.centre.x - sphere.radius;
            let sphere_x_max = sphere.centre.x + sphere.radius;

            let sphere_y_min = sphere.centre.y - sphere.radius;
            let sphere_y_max = sphere.centre.y + sphere.radius;

            let sphere_z_min = sphere.centre.z - sphere.radius;
            let sphere_z_max = sphere.centre.z + sphere.radius;

            if sphere_x_min < x_min {x_min = sphere_x_min}
            if sphere_x_max > x_max {x_max = sphere_x_max}
            
            if sphere_y_min < y_min {y_min = sphere_y_min}
            if sphere_y_max > y_max {y_max = sphere_y_max}

            if sphere_z_min < z_min {z_min = sphere_z_min}
            if sphere_z_max > z_max {z_max = sphere_z_max}
        }

        let width = x_max - x_min;
        let height = y_max - y_min;
        let depth = z_max - z_min;
        let corner = Vector3::new(x_min, y_min, z_min);

        BoundingBox {
            least_corner: corner,
            width,
            height,
            depth
        }
    }

    pub fn from_points(points: &Vec<Vector3>) -> BoundingBox{
        if points.len() == 0 {return BoundingBox::ZERO()}

        let mut x_min = points[0].x;
        let mut x_max = points[0].x;
        let mut y_min = points[0].y;
        let mut y_max = points[0].y;
        let mut z_min = points[0].z;
        let mut z_max = points[0].z;
         

        for point in points {
            if point.x < x_min {x_min = point.x}
            if point.x > x_max {x_max = point.x}

            if point.y < y_min {y_min = point.y}
            if point.y > y_max {y_max = point.y}

            if point.z < z_min {z_min = point.z}
            if point.z > z_max {z_max = point.z}
        }

        let corner = Vector3::new(x_min, y_min, z_min);
        let width = x_max - x_min;
        let height = y_max - y_min;
        let depth = z_max - z_min;

        BoundingBox{
            least_corner: corner,
            width,
            height,
            depth}
    }

    pub fn is_intersecting_box(&self, other: &BoundingBox) -> bool {
        self.least_corner.x <= other.least_corner.x + other.width &&
        self.least_corner.x + self.width >= other.least_corner.x &&
        self.least_corner.y <= other.least_corner.y + other.height &&
        self.least_corner.y + self.height >= other.least_corner.y &&
        self.least_corner.z <= other.least_corner.z + other.depth &&
        self.least_corner.z + self.width >= other.least_corner.z 
    }

}