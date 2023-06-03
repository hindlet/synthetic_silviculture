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

/// sorts the given values list by their accosiated f32 in ascending order
pub fn quicksort<T>(list: Vec<(f32, T)>) -> Vec<(f32, T)>{
    if list.len() <= 1 {return list;}

    let mut less = Vec::new();
    let mut equal = Vec::new();
    let mut more = Vec::new();

    let target_val = list[list.len() / 2].0;

    for item in list {
        if item.0 < target_val {less.push(item)}
        else if item > target_val {more.push(item)}
        else {equal.push(item)}
    }

    let mut sorted = quicksort(less);
    sorted.append(&mut equal);
    sorted.append(&mut quicksort(more));
    sorted
}

pub fn normal_probabilty_density(value: f32, mean: f32, standard_deviation: f32) -> f32 {

    (1.0 / (standard_deviation * ROOT_TWOPI)) * (-1.0 * (value - mean) * (value - mean) / (2.0 * standard_deviation * standard_deviation) ).exp()
}


#[cfg(test)]
mod quicksort_tests {
    use super::quicksort;
    #[test]
    fn random_test() {
        let list: Vec<(usize, f32)> = vec![(25.6, 0),(22.7, 1),(9.8, 2),(1.5, 3),(5.0, 4),(4.7, 5)];
        assert_eq!(quicksort(list), vec![(1.5, 3),(4.7, 5),(5.0, 4),(9.8, 2),(22.7, 1),(25.6, 0)]);
    }

    #[test]
    fn reverse_test() {
        let list: Vec<(usize, f32)> = vec![(25.6, 0),(22.7, 1),(9.8, 2),(5.0, 4),(4.7, 5),(1.5, 3)];
        assert_eq!(quicksort(list), vec![(1.5, 3),(4.7, 5),(5.0, 4),(9.8, 2),(22.7, 1),(25.6, 0)]);
    }

    #[test]
    fn pre_sorted_test() {
        let list: Vec<(usize, f32)>= vec![(1.5, 3),(4.7, 5),(5.0, 4),(9.8, 2),(22.7, 1),(25.6, 0)];
        assert_eq!(quicksort(list.clone()), list);
    }
}

#[test]
fn normal_dist_test() {
    assert!(0.048394144 - EPSILON < normal_probabilty_density(7.0, 12.0, 5.0) && normal_probabilty_density(7.0, 12.0, 5.0) < 0.048394144 + EPSILON);
    assert!(0.017205188 - EPSILON < normal_probabilty_density(56.0, 70.0, 15.0) && normal_probabilty_density(56.0, 70.0, 15.0) < 0.017205188 + EPSILON);
    assert!(0.002215924 - EPSILON < normal_probabilty_density(24.0, 18.0, 2.0) && normal_probabilty_density(24.0, 18.0, 2.0) < 0.002215924 + EPSILON);
    assert!(0.056068762 - EPSILON < normal_probabilty_density(19.2, 15.0, 5.0) && normal_probabilty_density(19.2, 15.0, 5.0) < 0.056068762 + EPSILON);
}



