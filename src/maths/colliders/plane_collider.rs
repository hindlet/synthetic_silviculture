use super::{Vector3, Vector2, Collider, RayHitInfo};


/// Axis Alligned Plane Collider
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct PlaneCollider {
    position: Vector3,
    x_length: f32,
    z_length: f32,
    centre: Vector3,
}

impl PlaneCollider {
    pub fn new(position: impl Into<Vector3>, size: impl Into<Vector2>) -> Self{
        let size = size.into();
        let position = position.into();
        PlaneCollider {
            position: position,
            x_length: size.x,
            z_length: size.y,
            centre: position + [size.x / 2.0, 0.0, size.y / 2.0].into()
        }
    }
}

impl Collider for PlaneCollider {
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: f32,
    ) -> Option<RayHitInfo> {
        let (root_position, direction): (Vector3, Vector3) = (root_position.into(), direction.into());
        let direction = direction.normalised();

        if (root_position == self.position) || ((self.centre - root_position).dot(Vector3::Y()) == 0.0) {
            return Some(RayHitInfo::new(root_position, 0.0));
        }

        if direction.dot(Vector3::Y()) == 0.0 {return None;}

        let distance = (self.centre - root_position).dot(Vector3::Y()) / direction.dot(Vector3::Y());

        if distance > max_distance {return None;}

        let point = root_position + direction * distance;

        if (point.x - self.position.x > self.x_length) || (point.z - self.position.z > self.z_length) {return None;}

        Some(RayHitInfo::new(point, distance))
    }
}

#[cfg(test)]
mod plane_collider_tests {
    use super::{PlaneCollider, Collider, RayHitInfo};

    #[test]
    fn parralel_ray_test() {
        let plane = PlaneCollider::new([0, 0, 0], [1, 1]);
        assert!(plane.check_ray([0, 1, 0], [1, 0, 0], 25.0).is_none())
    }

    #[test]
    fn position_ray_test() {
        let plane = PlaneCollider::new([0, 0, 0], [1, 1]);
        let hit = plane.check_ray([0, 0, 0], [1, 0, 0], 25.0).unwrap();
        assert_eq!(hit.hit_position, [0, 0, 0].into());
        assert_eq!(hit.hit_distance, 0.0);
    }

    #[test]
    fn contained_ray_test() {
        let plane = PlaneCollider::new([0, 0, 0], [5, 5]);
        let hit = plane.check_ray([1, 0, 1], [1, 0, 0], 25.0).unwrap();
        assert_eq!(hit.hit_position, [1, 0, 1].into());
        assert_eq!(hit.hit_distance, 0.0);
    }

    #[test]
    fn ray_hit_test() {
        let plane = PlaneCollider::new([0, 0, 0], [5, 5]);
        let hit = plane.check_ray([0, 5, 0], [0, -1, 0], 25.0).unwrap();
        assert_eq!(hit.hit_position, [0, 0, 0].into());
        assert_eq!(hit.hit_distance, 5.0);
    }
}