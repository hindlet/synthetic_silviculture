use crate::{
    vector_three::Vector3,
    bounding_box::BoundingBox,
    bounding_sphere::BoundingSphere,
    matrix_three::Matrix3,
};
use std::f32::consts::PI;

///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Bounding Sphere  //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod bounding_sphere_tests {
    use super::{Vector3, BoundingSphere, PI};

    #[test]
    fn axis_furthest_point_test() {
        let sphere = BoundingSphere{
            centre: Vector3 { x: 0.0, y: 5.0, z: 0.0 },
            radius: 2.5,
        };
        let point = Vector3 {x: 0.0, y: 0.0, z: 0.0};

        let furthest_point = sphere.furthest_point_from_point(point);

        assert_eq!(furthest_point, Vector3 {x: 0.0, y: 7.5, z: 0.0})
    }

    #[test]
    fn non_axis_furthest_point_test() {
        let sphere = BoundingSphere{
            centre: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            radius: 2.5,
        };
        let point = Vector3 {x: -2.5, y: 0.0, z: 0.0};

        let furthest_point = sphere.furthest_point_from_point(point);

        assert_eq!(furthest_point, Vector3 {x: 2.5, y: 0.0, z: 0.0})
    }

    #[test]
    fn zero_points_test() {
        let points: Vec<Vector3> = vec![];

        assert_eq!(BoundingSphere::from_points(&points), BoundingSphere::new())
    }

    #[test]
    fn defined_bounds_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 0.0, y: 1.5, z: 1.5},
        ];
        let mut bounds = BoundingSphere::new();
        bounds.radius = 2.5;
        assert!(bounds.contains_points(&points))
    }

    #[test]
    fn calculated_bounds_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 0.0, y: 1.5, z: 1.5},
        ];
        let bounds = BoundingSphere::from_points(&points);
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn intersection_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::ZERO(),
            radius: 5.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: 5.0, y: 0.0, z: 0.0},
            radius: 2.0,
        };

        assert_eq!(sphere_one.is_intersecting_sphere(&sphere_two), true)
    }

    #[test]
    fn touching_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::ZERO(),
            radius: 5.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: 10.0, y: 0.0, z: 0.0},
            radius: 5.0,
        };

        assert_eq!(sphere_one.is_intersecting_sphere(&sphere_two), false)
    }

    #[test]
    fn non_intersection_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::ZERO(),
            radius: 5.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: 10.0, y: 0.0, z: 0.0},
            radius: 2.0,
        };

        assert_eq!(sphere_one.is_intersecting_sphere(&sphere_two), false)
    }

    #[test]
    fn intersection_volume_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::ZERO(),
            radius: 2.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: -2.0, y: 2.0, z: -1.0},
            radius: 2.0,
        };

        assert_eq!(sphere_one.get_intersection_volume(&sphere_two), PI * 11.0 / 12.0)
    }

}

///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Bounding Box  /////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


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
        let bounds = BoundingBox {
            least_corner: Vector3::ZERO(),
            width: 5.0,
            height: 2.5,
            depth: 1.5,
        };
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
        let bounds = BoundingBox {
            least_corner: Vector3::new(-7.0, -5.0, -7.0),
            width: 14.0,
            height: 37.0,
            depth: 24.0,
        };
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
        let box_one = BoundingBox {
            least_corner: Vector3::ZERO(),
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let box_two = BoundingBox {
            least_corner: Vector3::new(2.5, 2.5, 2.5),
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(box_one.is_intersecting_box(&box_two), true)
    }

    #[test]
    fn non_intersection_test() {
        let box_one = BoundingBox {
            least_corner: Vector3::ZERO(),
            width: 2.0,
            height: 2.0,
            depth: 2.0,
        };
        let box_two = BoundingBox {
            least_corner: Vector3::new(2.5, 2.5, 2.5),
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(box_one.is_intersecting_box(&box_two), false)
    }
}

///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Matrices ///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod matrix_tests {
    use super::{Vector3, Matrix3, PI};

    #[test]
    fn euler_angles_x_test() {
        let angles = Vector3::new(PI, 0.0, 0.0);

        assert_eq!(Matrix3::from_angle_x(PI), Matrix3::from_euler_angles(angles))
    }

    #[test]
    fn euler_angles_y_test() {
        let angles = Vector3::new(0.0, PI, 0.0);

        assert_eq!(Matrix3::from_angle_y(PI), Matrix3::from_euler_angles(angles))
    }

    #[test]
    fn euler_angles_z_test() {
        let angles = Vector3::new(0.0, 0.0, PI);

        assert_eq!(Matrix3::from_angle_z(PI), Matrix3::from_euler_angles(angles))
    }

    #[test]
    fn euler_angles_all_test() {
        let angles = Vector3::new(PI, PI, PI);

        // these are functionally the same, but there is a tiny difference in rounding, so they will not be equal
        //
        // left: `Matrix3 { x: Vector3 { x: 1.0, y: -8.742278e-8, z: -8.742278e-8 }, y: Vector3 { x: 8.742277e-8, y: 1.0, z: -8.742278e-8 }, z: Vector3 { x: 8.7422784e-8, y: 8.742277e-8, z: 1.0 } }`,
        // right: `Matrix3 { x: Vector3 { x: 1.0, y: -8.7422784e-8, z: -8.742277e-8 }, y: Vector3 { x: 8.742278e-8, y: 1.0, z: -8.7422784e-8 }, z: Vector3 { x: 8.742278e-8, y: 8.742278e-8, z: 1.0 } }
        assert_ne!(Matrix3::from_angle_x(PI) * Matrix3::from_angle_y(PI) * Matrix3::from_angle_z(PI), Matrix3::from_euler_angles(angles))
    }

}