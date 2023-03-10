use std::{f32::consts::PI, ops::AddAssign};

use crate::{general::{vector_three::{self, Vector3}, matrix_three::Matrix3}, plant::{PlantData, PlantTag}, branch::{BranchTag, BranchConnectionData, BranchData, get_branches_base_to_tip}, branch_node::{BranchNodeConnectionData, BranchNodeTag, get_node_data_and_connections_base_to_tip, BranchNodeData}};
use bevy_ecs::prelude::*;

use super::{general_graphics::{Vertex, Normal}, branch_graphics::BranchGraphicsResources};

const HALF_PI: f32 = PI / 2.0;

// useful conversions and such for me here
impl Into<Vector3> for Vertex {
    fn into(self) -> Vector3 {
        Vector3::from(self.position)
    }
}

impl AddAssign<Vector3> for Normal {
    fn add_assign(&mut self, rhs: Vector3) {
        self.normal[0] += rhs.x;
        self.normal[1] += rhs.y;
        self.normal[2] += rhs.z;
    }
}

impl Normal {
    fn normalise(&mut self) {
        let mut normal = Vector3::from(self.normal);
        normal.normalise();
        self.normal = normal.into();
    }
}


#[derive(Component)]
pub struct MeshUpdateQueue (pub Vec<Entity>);


#[derive(Component)]
pub struct BranchMesh {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<u32>,
}

impl BranchMesh {
    pub fn empty() -> Self {
        BranchMesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new()
        }
    }

    pub fn set(&mut self, new: BranchMesh) {
        self.vertices = new.vertices;
        self.normals = new.normals;
        self.indices = new.indices;
    }
}


pub fn update_next_mesh(
    mut queue_qry: Query<&mut MeshUpdateQueue>,
    plants_query: Query<&PlantData, With<PlantTag>>,
    branch_data: Query<&BranchData, With<BranchTag>>,
    mut branch_meshes: Query<&mut BranchMesh, With<BranchTag>>,
    branch_connections: Query<&BranchConnectionData, With<BranchTag>>,
    node_connections: Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    node_data: Query<&BranchNodeData, With<BranchNodeTag>>,
    branch_graphics_res: Res<BranchGraphicsResources>,
) {
    let mut queue = queue_qry.single_mut();
    let polygons = &branch_graphics_res.polygon_vectors;

    loop {
        if queue.0.len() == 0 {return;}
        let id = queue.0[0];
        if let Ok(plant) = plants_query.get(id) {
            queue.0.remove(0);
            queue.0.push(id);
            if plant.root_node.is_none() {continue;}
            update_plant_mesh(&mut branch_meshes, &branch_data, &node_connections, &node_data, get_branches_base_to_tip(&branch_connections, plant.root_node.unwrap()), polygons);
            return;
        } else {queue.0.remove(0);}
    }

}


fn update_plant_mesh(
    branch_meshes: &mut Query<&mut BranchMesh, With<BranchTag>>,
    branch_data: &Query<&BranchData, With<BranchTag>>,
    node_connections: &Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
    node_data: &Query<&BranchNodeData, With<BranchNodeTag>>,
    branches: Vec<Entity>,
    polygon_directions: &Vec<Vector3>,
) {
    for id in branches.iter() {
        let new_mesh = {    
            if let Ok(branch) = branch_data.get(id.clone()) {
                if branch.root_node.is_none() {return;}
                let (pos, thick, connections) = get_node_data_and_connections_base_to_tip(node_connections, node_data, branch.root_node.unwrap());
                create_branch_mesh(pos, thick, connections, polygon_directions)
        } else {panic!("we are but pawns in a game of chess played by shrimp")}};
        if let Ok(mut mesh) = branch_meshes.get_mut(id.clone()) {
            mesh.set(new_mesh);
        }
    }
}

/// generates a mesh for a branch from node pairs and polygon directions
fn create_branch_mesh(
    node_pos: Vec<Vector3>,
    node_thicknesses: Vec<f32>,
    node_pairs:  Vec<(usize, usize)>,
    polygon_directions: &Vec<Vector3>,
) -> BranchMesh {

    

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut normals: Vec<Normal> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let num_indices = polygon_directions.len() as u32 * 2;

    for pair in node_pairs.iter() {
        let (node_1, node_2) = (node_pos[pair.0], node_pos[pair.1]);
        let (thick_1, thick_2) = (node_thicknesses[pair.0], node_thicknesses[pair.1]);

        let branch_line = node_2 - node_1;

        let rotation_matrix = {
            let mut rotation_axis = polygon_directions[0].cross(&branch_line);
            rotation_axis.normalise();
            let rotation_angle = polygon_directions[0].angle_to(&branch_line) - HALF_PI;
            Matrix3::from_angle_and_axis(rotation_angle, rotation_axis)
        };

        // add vertices and indices

        let vert_index = vertices.len() as u32;
        let mut incr: u32 = 0;
        for direction in polygon_directions.iter() {
            let new_dir = direction.clone().transform(rotation_matrix);
            vertices.push(Vertex{position: (node_1 + new_dir * thick_1).into()});
            vertices.push(Vertex{position: (node_2 + new_dir * thick_2).into()});

            // magic index stuff, this is just how it works, idk how else to explain it
            // it needed to loop round so that's where the mod comes in
            indices.push(vert_index + incr);
            indices.push(vert_index + incr + 1);
            indices.push(vert_index + 1 + (incr + 2) % num_indices);
            indices.push(vert_index + 1 + (incr + 2) % num_indices);
            indices.push(vert_index + (incr + 2) % num_indices);
            indices.push(vert_index + incr);
            incr += 2;
        }

        // init normals
        for _i in 0..vertices.len() {
            normals.push(Normal{normal: [0.0, 0.0, 0.0]})
        }
        
        // create normals
        for i in (0..indices.len()).step_by(3) {
            let dir_one: Vector3 = {
                let dir: Vector3 = vertices[indices[i + 1] as usize].into();
                dir - vertices[indices[i] as usize].into()
            };
            let dir_two: Vector3 = {
                let dir: Vector3 = vertices[indices[i + 2] as usize].into();
                dir - vertices[indices[i] as usize].into()
            };
            let normal = dir_one.cross(&dir_two);

            normals[indices[i + 0] as usize] += normal;
            normals[indices[i + 1] as usize] += normal;
            normals[indices[i + 2] as usize] += normal;
        }
    }

    // normalise normals
    for normal in normals.iter_mut() {
        normal.normalise();
    }

    // for vertex in vertices.iter() {
    //     println!("{:?}", vertex);
    // }

    BranchMesh {
        vertices,
        normals,
        indices
    }
}