use std::{f32::consts::PI, ops::AddAssign, collections::VecDeque};
use crate::{};
use bevy_ecs::{prelude::*, system::SystemState};
use super::{
    general_graphics::{PositionVertex, Normal},
    branch_graphics::BranchGraphicsResources,
    mesh::Mesh,
    super::{
        maths::{vector_three::{self, Vector3}, matrix_three::Matrix3},
        plants::plant::*,
        branches::{branch::*, branch_node::*},
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



#[derive(Component)]
pub struct MeshUpdateQueue {
    pub ids: VecDeque<Entity>,
    pub updates_per_frame: u32,
}

impl MeshUpdateQueue {
    pub fn new(updates_per_frame: u32) -> Self {
        MeshUpdateQueue{ids: VecDeque::new(), updates_per_frame}
    }

    pub fn new_from_single(id: Entity, updates_per_frame: u32) -> Self {
        MeshUpdateQueue{ids: VecDeque::from([id]), updates_per_frame}
    }

    pub fn new_from_many(ids: Vec<Entity>, updates_per_frame: u32) -> Self {
        MeshUpdateQueue{ids: VecDeque::from(ids), updates_per_frame}
    }
}


pub fn update_next_meshes(
    mut queue_qry: Query<&mut MeshUpdateQueue>,
    branch_data: Query<&BranchData, With<BranchTag>>,
    mut branch_meshes: Query<&mut Mesh, With<BranchTag>>,
    node_connections: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    node_data: Query<&BranchNodeData, With<BranchNodeTag>>,
    branch_graphics_res: Res<BranchGraphicsResources>,
) {
    let mut queue = queue_qry.single_mut();
    let polygons = &branch_graphics_res.polygon_vectors;
    let mut to_update = queue.updates_per_frame;

    loop {
        if queue.ids.len() == 0 {break;}
        if to_update == 0 {break;}
        let id = queue.ids.pop_front().unwrap();
        if update_branch_mesh(&mut branch_meshes, &branch_data, &node_connections, &node_data, id, polygons) {
            to_update -= 1;
        }
    }
}

pub fn update_all_meshes(
    mut queue_qry: Query<&mut MeshUpdateQueue>,
    branch_data: Query<&BranchData, With<BranchTag>>,
    mut branch_meshes: Query<&mut Mesh, With<BranchTag>>,
    node_connections: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    node_data: Query<&BranchNodeData, With<BranchNodeTag>>,
    branch_graphics_res: Res<BranchGraphicsResources>,
) {
    let mut queue = queue_qry.single_mut();
    let polygons = &branch_graphics_res.polygon_vectors;

    loop {
        if queue.ids.len() == 0 {break;}
        let id = queue.ids.pop_front().unwrap();
        update_branch_mesh(&mut branch_meshes, &branch_data, &node_connections, &node_data, id, polygons);
    }
}


pub fn check_for_force_update(
    branch_id_query: Query<Entity, With<BranchTag>>,
    branch_data: Query<&BranchData, With<BranchTag>>,
    mut branch_meshes: Query<&mut Mesh, With<BranchTag>>,
    node_connections: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    node_data: Query<&BranchNodeData, With<BranchNodeTag>>,
    branch_graphics_res: Res<BranchGraphicsResources>,
) {
    if branch_graphics_res.is_changed() {
        let polygons = &branch_graphics_res.polygon_vectors;
        for id in branch_id_query.iter() {
            update_branch_mesh(&mut branch_meshes, &branch_data, &node_connections, &node_data, id, polygons);
        }
    }
}


fn update_branch_mesh(
    branch_meshes: &mut Query<&mut Mesh, With<BranchTag>>,
    branch_data: &Query<&BranchData, With<BranchTag>>,
    node_connections: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    node_data: &Query<&BranchNodeData, With<BranchNodeTag>>,
    id: Entity,
    polygon_directions: &Vec<Vector3>,
) -> bool {
    let new_mesh = {    
        if let Ok(branch) = branch_data.get(id.clone()) {
            if branch.root_node.is_none() {return false;}
            let (pos, thick, connections) = get_node_data_and_connections_base_to_tip(node_connections, node_data, branch.root_node.unwrap());
            create_branch_mesh(branch.normal, branch.root_position, pos, thick, connections, polygon_directions)
        } else {return false;}
    };
    if let Ok(mut mesh) = branch_meshes.get_mut(id.clone()) {
        mesh.set(new_mesh);
        return true;
    }
    false
}

/// generates a mesh for a branch from node pairs and polygon directions
fn create_branch_mesh(
    branch_normal: Vector3,
    root_pos: Vector3,
    mut node_pos: Vec<Vector3>,
    node_thicknesses: Vec<f32>,
    node_pairs:  Vec<(usize, usize)>,
    polygon_directions: &Vec<Vector3>,
) -> Mesh {

    let branch_rotation_matrix = {
        let rotation_axis = branch_normal.cross(Vector3::Y());
        let rotation_angle = branch_normal.angle_to(Vector3::Y());
        Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
    };
    
    for node in node_pos.iter_mut() {
        node.mut_transform(branch_rotation_matrix);
    }   

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

    Mesh::new(vertices, indices).recalculate_normals().clone()
}