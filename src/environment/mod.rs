pub mod terrain;
pub mod light_cells;
use super::maths::vector_three::Vector3;




pub struct GravityResources{
    pub gravity_dir: Vector3,
    pub tropism_strength: f32, // positive for gravitropism, negative for phototropism
}


// PhysicalAgeStep
pub struct PhysicalAgeStep{
    pub step: f32,
}
