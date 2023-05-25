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

use std::f32::{
    consts::{FRAC_2_SQRT_PI, SQRT_2, PI},
    EPSILON
};

const ROOT_PI: f32 = FRAC_2_SQRT_PI * PI / 2.0; // sqrt(PI)
const ROOT_TWOPI: f32 = SQRT_2 * ROOT_PI; // sqrt(2.0 * PI)



pub fn lerp(start: f32, end: f32, position: f32) -> f32{
    start + (end - start) * position.clamp(0.0, 1.0)
}

pub fn normal_probabilty_density(value: f32, mean: f32, standard_deviation: f32) -> f32 {

    (1.0 / (standard_deviation * ROOT_TWOPI)) * (-1.0 * (value - mean) * (value - mean) / (2.0 * standard_deviation * standard_deviation) ).exp()
}

#[test]
fn normal_dist_test() {
    assert!(0.048394144 - EPSILON < normal_probabilty_density(7.0, 12.0, 5.0) && normal_probabilty_density(7.0, 12.0, 5.0) < 0.048394144 + EPSILON);
    assert!(0.017205188 - EPSILON < normal_probabilty_density(56.0, 70.0, 15.0) && normal_probabilty_density(56.0, 70.0, 15.0) < 0.017205188 + EPSILON);
    assert!(0.002215924 - EPSILON < normal_probabilty_density(24.0, 18.0, 2.0) && normal_probabilty_density(24.0, 18.0, 2.0) < 0.002215924 + EPSILON);
    assert!(0.056068762 - EPSILON < normal_probabilty_density(19.2, 15.0, 5.0) && normal_probabilty_density(19.2, 15.0, 5.0) < 0.056068762 + EPSILON);
}
