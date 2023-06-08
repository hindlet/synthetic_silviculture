pub mod terrain;
pub mod light_cells;
use super::maths::vector_three::Vector3;




pub struct GravitySettings{
    pub gravity_dir: Vector3,
    pub tropism_strength: f32, // positive for gravitropism, negative for phototropism
}

impl GravitySettings {
    pub fn create(direction: impl Into<Vector3>, strength: f32) -> Self{
        GravitySettings {
            gravity_dir: direction.into(),
            tropism_strength: strength
        }
    }
}

