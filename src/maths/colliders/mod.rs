use super::{vector_three::*, vector_two::Vector2, matrix_three::Matrix3, bounding_box::BoundingBox};

pub mod mesh_collider;
pub mod plane_collider;
pub mod triangle_collider;


pub trait Collider {
    fn check_ray(
        &self,
        root_position: impl Into<Vector3>,
        direction: impl Into<Vector3>,
        max_distance: Option<f32>,
    ) -> Option<RayHitInfo>;
}

#[derive(Debug)]
pub struct RayHitInfo {
    pub hit_position: Vector3,
    pub hit_distance: f32,
    pub hit_normal: Vector3,
}

impl RayHitInfo {
    pub fn new(position: Vector3, dist: f32, surface_normal: Vector3) -> Self{
        RayHitInfo{
            hit_position: position,
            hit_distance: dist,
            hit_normal: surface_normal
        }
    }
}