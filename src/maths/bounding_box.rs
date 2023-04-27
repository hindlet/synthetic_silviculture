#![allow(dead_code, unused_variables, unused_imports)]
use super::{vector_three::Vector3, bounding_sphere::BoundingSphere, colliders::{Collider, RayHitInfo}};


#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct BoundingBox {
    pub min_corner: Vector3, // least x y and z
    pub max_corner: Vector3,
}


impl BoundingBox {
    pub fn new(min_corner: Vector3, max_corner: Vector3) -> Self {
        BoundingBox {
            min_corner,
            max_corner
        }
    }

    #[allow(non_snake_case)]
    pub fn ZERO() -> Self {
        BoundingBox::new(Vector3::ZERO(), Vector3::ZERO())
    }


    pub fn contains_point(&self, point: &Vector3) -> bool {
        if (point.x < self.min_corner.x) || (point.x > self.max_corner.x) {
            return false
        }
        if (point.y < self.min_corner.y) || (point.y > self.max_corner.y) {
            return false
        }
        if (point.z < self.min_corner.z) || (point.z > self.max_corner.z) {
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

        if (sphere.centre.x - sphere.radius) < self.min_corner.x {return false;}
        if (sphere.centre.x + sphere.radius) > self.max_corner.x {return false;}

        if (sphere.centre.y - sphere.radius) < self.min_corner.y {return false;}
        if (sphere.centre.y + sphere.radius) > self.max_corner.y {return false;}

        if (sphere.centre.z - sphere.radius) < self.min_corner.z {return false;}
        if (sphere.centre.z + sphere.radius) > self.max_corner.z {return false;}

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

        let min_corner = Vector3::new(x_min, y_min, z_min);
        let max_corner =  Vector3::new(x_max, y_max, z_max);

        BoundingBox::new(min_corner, max_corner)
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

        let min_corner = Vector3::new(x_min, y_min, z_min);
        let max_corner = Vector3::new(x_max, y_max, z_max);

        BoundingBox::new(min_corner, max_corner)
    }

    pub fn is_intersecting_box(&self, other: &BoundingBox) -> bool {
        self.min_corner.x <= other.max_corner.x &&
        self.max_corner.x >= other.min_corner.x &&
        self.min_corner.y <= other.max_corner.y &&
        self.max_corner.y >= other.min_corner.y &&
        self.min_corner.z <= other.max_corner.z &&
        self.max_corner.z >= other.min_corner.z 
    }

}



impl Collider for BoundingBox {
    /// intersections with a box algorithm taken from Amy Williams et al. 2004
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: f32,
    ) -> Option<RayHitInfo> {
        let (root_position, direction): (Vector3, Vector3) = (root_position.into(), direction.into());
        let direction = direction.normalised();

        if self.contains_point(&root_position) {return Some(RayHitInfo::new(root_position, 0.0));}

        let inv_dir = Vector3::ONE() / direction;

        let (mut tmax, mut tmin) = if inv_dir.x < 0.0 {
            ((self.min_corner.x - root_position.x) * inv_dir.x, (self.max_corner.x - root_position.x) * inv_dir.x)
        } else {
            ((self.max_corner.x - root_position.x) * inv_dir.x, (self.min_corner.x - root_position.x) * inv_dir.x)
        };

        let (t_ymax, t_ymin) = if inv_dir.y < 0.0 {
            ((self.min_corner.y - root_position.y) * inv_dir.y, (self.max_corner.y - root_position.y) * inv_dir.y)
        } else {
            ((self.max_corner.y - root_position.y) * inv_dir.y, (self.min_corner.y - root_position.y) * inv_dir.y)
        };


        if (tmin > t_ymax) || (t_ymin > tmax) {return None;}

        if t_ymin > tmin {tmin = t_ymin}
        if t_ymax < tmax {tmax = t_ymax}

        let (t_zmax, t_zmin) = if inv_dir.z < 0.0 {
            ((self.min_corner.z - root_position.z) * inv_dir.z, (self.max_corner.z - root_position.z) * inv_dir.z)
        } else {
            ((self.max_corner.z - root_position.z) * inv_dir.z, (self.min_corner.z - root_position.z) * inv_dir.z)
        };

        if (tmin > t_zmax) || (t_zmin > tmax) {return None;}

        if t_zmin > tmin {tmin = t_zmin}
        if t_zmax < tmax {tmax = t_zmax}

        let dist = if tmin < 0.0 {tmax} else if tmax < 0.0 {return None} else {tmin};

        Some(RayHitInfo::new(root_position + direction * dist, dist))
    }
}




#[cfg(test)]
mod bounding_box_tests {
    use super::{Vector3, BoundingBox, BoundingSphere};

    #[test]
    fn zero_points_test() {
        let points: Vec<Vector3> = vec![];

        assert_eq!(BoundingBox::from_points(&points), BoundingBox::ZERO())
    }

    #[test]
    fn defined_bounds_point_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 5.0, y: 1.5, z: 1.5},
        ];
        let bounds = BoundingBox::new(Vector3::ZERO(), Vector3::new(5.0, 2.5, 1.5));
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn calculated_bounds_point_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 5.0, y: 1.5, z: 1.5},
        ];
        let bounds = BoundingBox::from_points(&points);
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn defined_bounds_sphere_test() {
        let spheres: Vec<BoundingSphere> = vec![
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 0.0, z: 0.0},
                radius: 5.0,
            },
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 25.0, z:0.0},
                radius: 7.0,
            },
            BoundingSphere {
                centre: Vector3{x: 5.0, y: 15.0, z: 15.0},
                radius: 2.0,
            },
        ];
        let bounds = BoundingBox::new(Vector3::new(-7.0, -5.0, -7.0), Vector3::new(7.0, 32.0, 17.0));
        assert_eq!(bounds.contains_spheres(&spheres), true)
    }

    #[test]
    fn calculated_bounds_sphere_test() {
        let spheres: Vec<BoundingSphere> = vec![
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 0.0, z: 0.0},
                radius: 5.0,
            },
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 25.0, z:0.0},
                radius: 7.0,
            },
            BoundingSphere {
                centre: Vector3{x: 5.0, y: 15.0, z: 15.0},
                radius: 2.0,
            },
        ];
        let bounds = BoundingBox::from_spheres(&spheres);

        assert_eq!(bounds.contains_spheres(&spheres), true)
    }

    #[test]
    fn intersection_test() {
        let box_one = BoundingBox::new(Vector3::ZERO(), Vector3::ONE() * 5.0);
        let box_two = BoundingBox::new(Vector3::ONE() * 2.5, Vector3::ONE() * 5.0);
        assert_eq!(box_one.is_intersecting_box(&box_two), true)
    }

    #[test]
    fn non_intersection_test() {
        let box_one = BoundingBox::new(Vector3::ZERO(), Vector3::ONE() * 2.0);
        let box_two = BoundingBox::new(Vector3::ONE() * 2.5, Vector3::ONE() * 5.0);
        assert_eq!(box_one.is_intersecting_box(&box_two), false)
    }

}


#[cfg(test)]
mod bounding_box_collider_tests {
    use super::{Vector3, BoundingBox, Collider};
    #[test]
    fn perpendicular_intersection_test() {
        let bounds = BoundingBox::new(Vector3::ZERO(), Vector3::ONE() * 5.0);
        let hit = bounds.check_ray([10, 2, 0], -Vector3::X(), 25.0).unwrap();
        assert_eq!(hit.hit_position, [5, 2, 0].into());
        assert_eq!(hit.hit_distance, 5.0);
    }

    #[test]
    fn contained_intersection_test() {
        let bounds = BoundingBox::new(Vector3::ZERO(), Vector3::ONE() * 5.0);
        let hit = bounds.check_ray([3, 2, 0], -Vector3::X(), 25.0).unwrap();
        assert_eq!(hit.hit_position, [3, 2, 0].into());
        assert_eq!(hit.hit_distance, 0.0);
    }

    #[test]
    fn angled_test() {
        let bounds = BoundingBox::new(Vector3::ZERO(), Vector3::ONE() * 5.0);
        let hit = bounds.check_ray([8, 2, 0], [-1, 0, 1], 25.0).unwrap();
        assert_eq!(hit.hit_position, [5.0, 2.0, 2.9999998].into()); // this should be (5, 2, 3) but due to rounding errors from floats the last number is lightly off, within acceptable range
        assert_eq!(hit.hit_distance, 3.0 * 2_f32.sqrt());
    }
}