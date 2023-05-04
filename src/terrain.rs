use bevy_ecs::prelude::*;
use image::{DynamicImage, GenericImageView};
use super::{
    maths::{
        colliders::{Collider, plane_collider::PlaneCollider, mesh_collider::MeshCollider},
        vector_three::Vector3, vector_two::Vector2,
    },
    graphics::{
        mesh::Mesh,
    }
};



#[derive(Component)]
pub struct TerrainTag;

/// Wrapper for a Collider
#[derive(Component)]
pub struct TerrainCollider<T: Collider> {
    collider: T
}


#[derive(Bundle)]
pub struct HeightMapTerrainBundle {
    tag: TerrainTag,
    collider: TerrainCollider<MeshCollider>,
    mesh: Mesh
}


#[derive(Bundle)]
pub struct FlatTerrainBundle {
    tag: TerrainTag,
    collider: TerrainCollider<PlaneCollider>,
    mesh: Mesh
}




/// this will spawn terrain from a given heightmap
pub fn spawn_heightmap_terrain(
    size: f32,
    vertices_per_side: u32,
    height_scale: f32,
    centre: impl Into<Vector3>,
    heightmap_path: &str,
    world: &mut World,
) {
    let size = size.max(0.000000001);
    let vertices_per_side = vertices_per_side.max(2);
    let centre: Vector3 = centre.into();
    let (start_x, start_y) = (centre.x - size / 2.0, centre.z - size / 2.0);

    let tri_size = size / vertices_per_side as f32;
    let heightmap = image::open(heightmap_path).unwrap();
    let (x_step, y_step) = (heightmap.width() / vertices_per_side, heightmap.height() / vertices_per_side);

    let mut vertices: Vec<Vector3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for x in 0..vertices_per_side {
        for y in 0..vertices_per_side {
            let height = (heightmap.get_pixel(x * x_step, y * y_step).0[0] as f32 / 255.0) * height_scale;
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

    
    let mesh: Mesh = Mesh::from((vertices.clone(), indices.clone())).recalculate_normals().clone();
    world.spawn(
        HeightMapTerrainBundle{
            tag: TerrainTag,
            collider: TerrainCollider{collider: MeshCollider::new(vertices, indices)},
            mesh
        }
    );
}


pub fn spawn_flat_terrain(
    size: f32,
    centre: impl Into<Vector3>,
    world: &mut World,
) {
    let size = size.max(0.000000001);
    let centre: Vector3 = centre.into();

    let vertices: Vec<Vector3> = vec![
        [centre.x - size / 2.0, centre.y, centre.z - size / 2.0].into(),
        [centre.x - size / 2.0, centre.y, centre.z + size / 2.0].into(),
        [centre.x + size / 2.0, centre.y, centre.z - size / 2.0].into(),
        [centre.x + size / 2.0, centre.y, centre.z + size / 2.0].into()
    ];
    let indices: Vec<u32> = vec![
        0, 3, 2, 3, 0, 1
    ];

    let mesh: Mesh = Mesh::from((vertices.clone(), indices.clone())).recalculate_normals().clone();

    world.spawn(
        FlatTerrainBundle{
            tag: TerrainTag,
            collider: TerrainCollider { collider: PlaneCollider::new([centre.x, centre.y, centre.z], [size, size]) },
            mesh
        }
    );
}