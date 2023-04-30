// tests
pub mod tests;
// vectors
pub mod vector_two;
pub mod vector_three;
pub mod vector_three_int;
pub mod vector_four;
// matrices
pub mod matrix_three;
pub mod matrix_four;
// bounds
pub mod bounding_sphere;
pub mod bounding_box;
// colliders
pub mod colliders;



pub fn lerp(start: f32, end: f32, position: f32) -> f32{
    start + (end - start) * position.clamp(0.0, 1.0)
}

