#![allow(dead_code, unused_variables, unused_imports)]
use super::{Vector3, Collider, RayHitInfo, triangle_collider::TriangleCollider, BoundingBox};


#[derive(Default, Debug, PartialEq, Clone)]
pub struct MeshCollider {
    tris: Vec<TriangleCollider>,
    bounds: BoundingBox
}


/// basic quicksort function
fn quicksort(unsorted: Vec<(usize, f32)>) -> Vec<(usize, f32)>{
    if unsorted.len() <= 1 {return unsorted;}

    let mid = unsorted[unsorted.len() / 2];

    let mut lower = Vec::new();
    let mut higher = Vec::new();
    let mut equal = Vec::new();
    for val in unsorted {
        if val.1 < mid.1 {lower.push(val)}
        else if val.1 > mid.1 {higher.push(val)}
        else {equal.push(val)}
    }

    let mut result = quicksort(lower);
    result.append(&mut equal);
    result.append(&mut quicksort(higher));
    result
}


impl MeshCollider {
    pub fn new(vertices: Vec<Vector3>, indices: Vec<u32>) -> Self{
        let mut tris = Vec::new();

        for i in (0..indices.len()).step_by(3) {
            tris.push(TriangleCollider::new(vertices[indices[i] as usize], vertices[indices[i + 1] as usize], vertices[indices[i + 2] as usize]));
        }

        let bounds = BoundingBox::from_points(&vertices);
        MeshCollider {
            tris,
            bounds
        }
    }
}

impl Collider for MeshCollider {
    /// for checking intersections of a mesh and ray, I chose to sort the mesh triangles by distance from the ray root and loop through them
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: f32,
    ) -> Option<RayHitInfo> {
        let (root_position, direction): (Vector3, Vector3) = (root_position.into(), direction.into());
        let direction = direction.normalised();

        // sort the tris by distance
        let dist_indices = {
            let mut indices = Vec::new();
            for i in 0..self.tris.len() {
                indices.push((i, self.tris[i].centre_dist_to(root_position)));
            }
            indices
        };

        let sorted_indices = quicksort(dist_indices);
        for index in sorted_indices {
            let check = self.tris[index.0].check_ray(root_position, direction, max_distance);
            if check.is_some() {return check;}
        }
        None
    }
}


#[cfg(test)]
mod mesh_quicksort_tests {
    use super::quicksort;
    #[test]
    fn random_test() {
        let list: Vec<(usize, f32)> = vec![(0, 25.6),(1, 22.7),(2, 9.8),(3, 1.5),(4, 5.0),(5, 4.7)];
        assert_eq!(quicksort(list), vec![(3, 1.5),(5, 4.7),(4, 5.0),(2, 9.8),(1, 22.7),(0, 25.6)]);
    }

    #[test]
    fn reverse_test() {
        let list: Vec<(usize, f32)> = vec![(0, 25.6),(1, 22.7),(2, 9.8),(4, 5.0),(5, 4.7),(3, 1.5)];
        assert_eq!(quicksort(list), vec![(3, 1.5),(5, 4.7),(4, 5.0),(2, 9.8),(1, 22.7),(0, 25.6)]);
    }

    #[test]
    fn pre_sorted_test() {
        let list: Vec<(usize, f32)>= vec![(3, 1.5),(5, 4.7),(4, 5.0),(2, 9.8),(1, 22.7),(0, 25.6)];
        assert_eq!(quicksort(list.clone()), list);
    }
}