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

pub struct RayHitInfo {
    pub hit_position: Vector3,
    pub hit_distance: f32,
}

impl RayHitInfo {
    pub fn new(position: Vector3, dist: f32) -> Self{
        RayHitInfo{
            hit_position: position,
            hit_distance: dist
        }
    }
}


pub fn check_ray_collision(
    root_position: Vector3,
    direction: Vector3,
    max_distance: Option<f32>,
    collider: impl Collider
) -> Option<RayHitInfo>{
    collider.check_ray(root_position, direction, max_distance)
}