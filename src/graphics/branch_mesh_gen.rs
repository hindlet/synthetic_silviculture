use std::{f32::consts::PI, ops::AddAssign, collections::VecDeque};
use crate::{};
use bevy_ecs::{prelude::*, system::SystemState};
use super::{
    general_graphics::{PositionVertex, Normal},
    branch_graphics::BranchGraphicsResources,
    mesh::Mesh,
    super::{
        maths::{vector_three::{self, Vector3}, matrix_three::Matrix3, quicksort},
        plants::plant::*,
        branches::{branch::*, branch_node::*, branch_sorting::get_branch_mesh_update_times, node_sorting::get_node_data_and_connections_base_to_tip},
    }
};


// useful conversions and such for me here
impl Into<Vector3> for PositionVertex {
    fn into(self) -> Vector3 {
        Vector3::from(self.position)
    }
}

impl AddAssign<Vector3> for PositionVertex {
    fn add_assign(&mut self, rhs: Vector3) {
        self.position[0] += rhs.x;
        self.position[1] += rhs.y;
        self.position[2] += rhs.z;
    }
}

impl AddAssign<Vector3> for Normal {
    fn add_assign(&mut self, rhs: Vector3) {
        self.normal[0] += rhs.x;
        self.normal[1] += rhs.y;
        self.normal[2] += rhs.z;
    }
}



pub fn update_next_mesh(
    plants: &Vec<Plant>,
    branch_graphics_res: &BranchGraphicsResources,
) {
    let polygons = &branch_graphics_res.polygon_vectors;

    let unsorted_branch_updates = get_branch_mesh_update_times(plants);

    if unsorted_branch_updates.len() == 0 {return;}

    if unsorted_branch_updates.len() == 1 {
        let (positions, radii, pairs) = get_node_data_and_connections_base_to_tip(&unsorted_branch_updates[0].1.root);
        unsorted_branch_updates[0].1.mesh = create_branch_mesh(unsorted_branch_updates[0].1.data.root_position, positions, radii, pairs, polygons);
        return;
    }

    let branch = quicksort(unsorted_branch_updates)[0].1;
    let (positions, radii, pairs) = get_node_data_and_connections_base_to_tip(&branch.root);
    branch.mesh = create_branch_mesh(branch.data.root_position, positions, radii, pairs, polygons);
    return;


}


// pub fn check_for_force_update(
    
//     branch_graphics_res: &BranchGraphicsResources,
// ) {
//     if branch_graphics_res.is_changed() {
//         let polygons = &branch_graphics_res.polygon_vectors;
//         for id in branch_id_query.iter() {
//             update_branch_mesh(&mut branch_meshes, &branch_data, &node_connections, &node_data, id, polygons);
//         }
//     }
// }


/// generates a mesh for a branch from node pairs and polygon directions
fn create_branch_mesh(
    root_pos: Vector3,
    node_pos: Vec<Vector3>,
    node_thicknesses: Vec<f32>,
    node_pairs:  Vec<(usize, usize)>,
    polygon_directions: &Vec<Vector3>,
) -> Mesh {

    let mut vertices: Vec<PositionVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let num_indices = polygon_directions.len() as u32 * 2;

    for pair in node_pairs.iter() {
        let (node_1, node_2) = (node_pos[pair.0], node_pos[pair.1]);
        let (thick_1, thick_2) = (node_thicknesses[pair.0], node_thicknesses[pair.1]);

        let mut branch_line = node_2 - node_1;
        branch_line.normalise();

        let allignment_mat = {
            let mut rotation_axis = branch_line.cross(Vector3::Y());
            rotation_axis.normalise();
            let rotation_angle = Vector3::Y().angle_to(branch_line);
            Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
        };

        // add vertices and indices
        let vert_index = vertices.len() as u32;
        let mut incr: u32 = 0;
        for direction in polygon_directions.iter() {
            let new_dir = direction.transform(allignment_mat);
            vertices.push(PositionVertex{position: (node_1 + (new_dir * thick_1)).into()});
            vertices.push(PositionVertex{position: (node_2 + (new_dir * thick_2)).into()});

            // magic index stuff, this is just how it works, idk how else to explain it
            // it needed to loop round so that's where the mod comes in
            indices.push(vert_index + incr);
            indices.push(vert_index + 1 + (incr + 2) % num_indices);
            indices.push(vert_index + (incr + 2) % num_indices);
            indices.push(vert_index + 1 + (incr + 2) % num_indices);
            indices.push(vert_index + incr);
            indices.push(vert_index + incr + 1);
            incr += 2;
        }

        
    }

    // move mesh to root
    for vertex in vertices.iter_mut() {
        *vertex += root_pos;
    }

    Mesh::new(vertices, indices).recalculate_normals()
}