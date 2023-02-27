use crate::branch;
use crate::vector_three::Vector3;
use crate::matrix_three::Matrix3;
use std::f32::consts::PI;

mod branch_vertex_compute_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "assets/shaders/branch_vertex_compute.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}


/// Creates a polygon of vectors,
/// each of the vectors is a direction 
/// so can be used to generate a polygon from a central point. 
/// By default, the first direction will go along the x axis. 
/// A rotation will rotate by that many radians anticlockwise about the y axis
fn create_vector_polygon(sides: u32, rotation: Option<f32>) -> Vec<Vector3> {
    let inner_rotation_matrix = Matrix3::from_angle_y(PI / sides as f32);
    let initial_direction = if rotation.is_none() {
        Vector3::X()
    } else {
        Vector3::X().transform(Matrix3::from_angle_y(rotation.unwrap()))
    };
    let mut vectors = vec![initial_direction];
    for i in 0..(sides-1) as usize {
        vectors.push(vectors[i].clone().transform(inner_rotation_matrix));
    }
    vectors
}


