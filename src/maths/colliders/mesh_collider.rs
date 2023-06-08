#![allow(dead_code, unused_variables, unused_imports)]
use super::{Vector3, Collider, RayHitInfo, triangle_collider::TriangleCollider, BoundingBox, quicksort};


#[derive(Default, Debug, PartialEq, Clone)]
pub struct MeshCollider {
    tris: Vec<TriangleCollider>,
    bounds: BoundingBox
}





impl MeshCollider {
    pub fn new(vertices: Vec<Vector3>, indices: Vec<u32>) -> Self{
        let mut tris = Vec::new();

        for i in (0..indices.len()).step_by(3) {
            tris.push(TriangleCollider::new(vertices[indices[i] as usize], vertices[indices[i + 1] as usize], vertices[indices[i + 2] as usize]));
        }

        let bounds = BoundingBox::from_points(vertices);
        MeshCollider {
            tris,
            bounds
        }
    }
}

impl Collider for MeshCollider {
    /// for checking intersections of a mesh and ray, I chose to sort the mesh triangles by distance from the ray root and loop through them
    /// 
    /// A mesh does not check if a point is contained as there is not guarantee there is an inside to a mesh
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: Option<f32>,
    ) -> Option<RayHitInfo> {
        let (root_position, direction): (Vector3, Vector3) = (root_position.into(), direction.into());
        let direction = direction.normalised();

        if self.bounds.check_ray(root_position, direction, max_distance).is_none() {return None;}

        // sort the tris by distance
        let dist_indices = {
            let mut indices = Vec::new();
            for i in 0..self.tris.len() {
                indices.push((self.tris[i].centre_dist_to(root_position), i));
            }
            indices
        };

        let sorted_indices = quicksort(dist_indices);
        for index in sorted_indices {
            let check = self.tris[index.1].check_ray(root_position, direction, max_distance);
            if check.is_some() {return check;}
        }
        None
    }
}


#[cfg(test)]
mod mesh_collider_tests {
    use super::{MeshCollider, Collider};

    #[test]
    fn miss_mesh_test() {
        let mesh = MeshCollider::new(
            vec![[-2, 0, -2].into(), [-2, 0, 2].into(), [2, 0, -2].into(), [2, 0, 2].into(), [0, 5, 0].into()],
            vec![0, 3, 2, 3, 1, 0, 0, 4, 1, 1, 4, 2, 2, 4, 3, 3, 4, 1]
        );

        assert!(mesh.check_ray([1, 4, 0], [1, 0, 0], Some(25.0)).is_none())
    }

    #[test]
    fn miss_box_test() {
        let mesh = MeshCollider::new(
            vec![[-2, 0, -2].into(), [-2, 0, 2].into(), [2, 0, -2].into(), [2, 0, 2].into(), [0, 5, 0].into()],
            vec![0, 3, 2, 3, 1, 0, 0, 4, 1, 1, 4, 2, 2, 4, 3, 3, 4, 1]
        );

        assert!(mesh.check_ray([1, 6, 0], [1, 0, 0], Some(25.0)).is_none())
    }

    #[test]
    fn hit_test() {
        let mesh = MeshCollider::new(
            vec![[-2, 0, -2].into(), [-2, 0, 2].into(), [2, 0, -2].into(), [2, 0, 2].into(), [-2, 5, 0].into()],
            vec![0, 3, 2, 3, 1, 0, 0, 4, 1, 1, 4, 2, 2, 4, 3, 3, 4, 1]
        );

        let hit = mesh.check_ray([-5, 3, 0], [1, 0, 0], Some(25.0)).unwrap();

        assert_eq!(hit.hit_position, [-2, 3, 0].into());
        assert_eq!(hit.hit_normal, [0, 1, 0].into());
        assert_eq!(hit.hit_distance, 3.0);
    }

    #[test]
    fn as_plane_test() {
        let collider = MeshCollider::new(
            vec![
            [-25.0, 0.0, -25.0].into(),
            [-25.0, 0.0, 25.0].into(),
            [25.0, 0.0, -25.0].into(),
            [25.0, 0.0, 25.0].into()],
            vec![0, 3, 2, 3, 0, 1]
        );

        let hit = collider.check_ray([5, 10, 5], [0, -1, 0], None).unwrap();

        assert_eq!(hit.hit_position, [5, 0, 5].into());
        assert_eq!(hit.hit_normal, [0, 1, 0].into());
        assert_eq!(hit.hit_distance, 10.0);
    }
}