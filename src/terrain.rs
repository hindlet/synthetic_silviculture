use bevy_ecs::prelude::*;
use super::{
    maths::{
        colliders::{Collider, plane_collider::PlaneCollider, mesh_collider::MeshCollider},
        vector_three::Vector3
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

#[derive(Resource)]
pub struct TerrainMeshBuffers {
    pub vertices: Subbuffer<[PositionVertex]>,
    pub normals: Subbuffer<[Normal]>,
    pub indices: Subbuffer<[u32]>,
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



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// Graphics ////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////
