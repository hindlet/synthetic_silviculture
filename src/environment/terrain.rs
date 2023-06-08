use image::{DynamicImage, GenericImageView};
use std::ops::RangeInclusive;
use super::super::{
    maths::{
        colliders::{RayHitInfo, Collider, mesh_collider::MeshCollider},
        vector_three::Vector3, vector_two::Vector2,
    },
    graphics::{
        mesh::Mesh,
    }
};


pub struct Terrain{
    collider: MeshCollider,
    pub mesh: Mesh
}


impl Terrain {
    pub fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: Option<f32>,
    ) -> Option<RayHitInfo> {
        self.collider.check_ray(root_position, direction, max_distance)
    }
}



/// this will spawn terrain from a given heightmap
pub fn create_heightmap_terrain(
    size: f32,
    vertices_per_side: u32,
    height_scale: f32,
    centre: impl Into<Vector3>,
    heightmap_path: String,
) -> (Terrain, (f32, RangeInclusive<f32>, RangeInclusive<f32>)) {
    let size = size.max(0.000000001);
    let vertices_per_side = vertices_per_side.max(2);
    let centre: Vector3 = centre.into();
    let (start_x, start_y) = (centre.x - size / 2.0, centre.z - size / 2.0);

    let tri_size = size / vertices_per_side as f32;
    let heightmap = image::open(heightmap_path).unwrap();
    let (x_step, y_step) = (heightmap.width() / vertices_per_side, heightmap.height() / vertices_per_side);

    let mut vertices: Vec<Vector3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut max_height = -10.0;

    for x in 0..vertices_per_side {
        for y in 0..vertices_per_side {
            let height = (heightmap.get_pixel(x * x_step, y * y_step).0[0] as f32 / 255.0) * height_scale;
            if height > max_height {max_height = height};
            vertices.push([start_x + x as f32 * tri_size, height + centre.y, start_y + y as f32 * tri_size].into());
            if x < vertices_per_side - 1 && y < vertices_per_side - 1 {
                indices.push(vertices.len() as u32 - 1);
                indices.push(vertices.len() as u32 + vertices_per_side);
                indices.push(vertices.len() as u32 + vertices_per_side - 1);
                indices.push(vertices.len() as u32 + vertices_per_side);
                indices.push(vertices.len() as u32 - 1);
                indices.push(vertices.len() as u32);
                
            }
        }
    }

    let collider = MeshCollider::new(vertices.clone(), indices.clone());

    
    let mesh: Mesh = Mesh::from((vertices.clone(), indices.clone())).recalculate_normals().clone();
    

    (Terrain{collider: collider, mesh: mesh}, (max_height, (centre.x - size)..=(centre.x + size), (centre.z - size)..=(centre.z + size)))
}


pub fn create_flat_terrain(
    size: f32,
    centre: impl Into<Vector3>,
) -> (Terrain, (f32, RangeInclusive<f32>, RangeInclusive<f32>)){
    let size = size.max(0.000000001);
    let centre: Vector3 = centre.into();
    let half_size = size / 2.0;

    let vertices: Vec<Vector3> = vec![
        [centre.x - half_size, centre.y, centre.z - half_size].into(),
        [centre.x - half_size, centre.y, centre.z + half_size].into(),
        [centre.x + half_size, centre.y, centre.z - half_size].into(),
        [centre.x + half_size, centre.y, centre.z + half_size].into()
    ];
    let indices: Vec<u32> = vec![
        0, 3, 2, 3, 0, 1
    ];

    let mesh: Mesh = Mesh::from((vertices.clone(), indices.clone())).recalculate_normals().clone();
    let collider = MeshCollider::new(vertices, indices);


    (Terrain{collider: collider, mesh: mesh}, (centre.y, (centre.x - half_size)..=(centre.x + half_size), (centre.z - half_size)..=(centre.z + half_size)))
}