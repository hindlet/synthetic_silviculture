use super::{Vector3, Vector2, Collider, RayHitInfo, Matrix3, cross};


#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct TriangleCollider {
    normal: Vector3,
    points: Matrix3,

    edge_one: Vector3,
    edge_two: Vector3,

    centre: Vector3,
}

impl TriangleCollider {
    // creates a new triangle collider from the given points
    pub fn new(a: impl Into<Vector3>, b: impl Into<Vector3>, c: impl Into<Vector3>) -> Self{
        let (a, b, c) = (a.into(), b.into(), c.into());
        let centre = compute_incentre(a, b, c);
        let (edge_one, edge_two) = (b - a,c - a);
        let normal = cross(edge_one, edge_two).normalised();
        TriangleCollider {
            normal,
            points: Matrix3::from_columns(a, b, c),
            edge_one,
            edge_two,
            centre
        }
    }

    pub fn centre_dist_to(&self, target: Vector3) -> f32{
        (self.centre - target).magnitude()
    }

}

fn compute_incentre(a: Vector3, b: Vector3, c: Vector3) -> Vector3 {
        
    let p = (b-c).magnitude();
    let q = (c-a).magnitude();
    let r = (a-b).magnitude();
    let sum = p + q + r;

    let centre = a * (p/sum) + b * (q/sum) + c * (r/sum);

    centre
}



impl Collider for TriangleCollider {
    /// this uses the [Möller-Trumbore algorithm](https://en.wikipedia.org/wiki/Möller–Trumbore_intersection_algorithm) for ray-triangle intersections
    /// current known issue that if ray start is in triangle but direction is parallel will fail
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: Option<f32>,
    ) -> Option<RayHitInfo> {
        let (root_position, direction): (Vector3, Vector3) = (root_position.into(), direction.into());
        let direction = direction.normalised();

        let h = cross(direction, self.edge_two);
        let a = h.dot(self.edge_one);

        if a == 0.0 {
            // println!("1 fail");
            return None;
        } // ray is parallel to triangle

        let f = 1.0 / a;
        let s = root_position - self.points.c1();
        let u = f * s.dot(h);

        if (u < 0.0) || (u > 1.0) {
            // println!("2 fail");
            return None;
        }

        let q = cross(s, self.edge_one);
        let v = f * direction.dot(q);

        if (v < 0.0) || (u + v > 1.0) {
            // println!("3 fail");
            return None;
        }

        let t = f * self.edge_two.dot(q);

        if (max_distance.is_some() && t > max_distance.unwrap()) || (t < 0.0) {return None;}

        Some(RayHitInfo::new(root_position + direction * t, t))
    }
}


#[cfg(test)]
mod triangle_collider_tests {
    use super::{TriangleCollider, Collider, RayHitInfo};

    #[test]
    fn parralel_ray_test() {
        let tri = TriangleCollider::new([0, 0, 0], [1, 0, 0], [0, 0, 1]);
        assert!(tri.check_ray([0, 1, 0], [1, 0, 0], Some(25.0)).is_none());
    }

    #[test]
    fn position_ray_test() {
        let tri = TriangleCollider::new([0, 0, 0], [1, 0, 0], [0, 0, 1]);
        let hit = tri.check_ray([0, 0, 0], [0, 1, 0], Some(25.0)).unwrap();
        assert_eq!(hit.hit_position, [0, 0, 0].into());
        assert_eq!(hit.hit_distance, 0.0);
    }

    #[test]
    fn contained_ray_test() {
        let tri = TriangleCollider::new([0, 0, 0], [1, 0, 0], [0, 0, 1]);
        let hit = tri.check_ray([0.25, 0.0, 0.25], [0, 1, 0], Some(25.0)).unwrap();
        assert_eq!(hit.hit_position, [0.25, 0.0, 0.25].into());
        assert_eq!(hit.hit_distance, 0.0);
    }

    #[test]
    fn ray_hit_test() {
        let tri = TriangleCollider::new([0, 0, 0], [1, 0, 0], [0, 0, 1]);
        let hit = tri.check_ray([0, 5, 0], [0, -1, 0], Some(25.0)).unwrap();
        assert_eq!(hit.hit_position, [0, 0, 0].into());
        assert_eq!(hit.hit_distance, 5.0);
    }

    #[test]
    fn centre_test() {
        let tri = TriangleCollider::new([-2, 0, -1], [2, 0, -1], [2, 0, 1]);

        // the answer given for the first is false however that is due to small rounding errors and is within acceptable error
        // assert_eq!(tri.centre, [5_f32.sqrt() - 1.0, 0.0, 2.0 - 5_f32.sqrt()].into())
        assert_eq!(tri.centre, [1.236068, 0.0, -0.23606798].into())
    }
}