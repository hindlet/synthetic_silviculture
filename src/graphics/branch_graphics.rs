use crate::branch::*;
use crate::branch_node::*;
use crate::plant::PlantData;
use crate::plant::PlantTag;
use crate::vector_three::Vector3;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use itertools::Itertools;
use vulkano::DeviceSize;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::DeviceLocalBuffer;
use vulkano::buffer::TypedBufferAccess;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferInheritanceInfo;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::DescriptorSetWithOffsets;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::memory::allocator::FreeListAllocator;
use vulkano::memory::allocator::GenericMemoryAllocator;
use vulkano::memory::allocator::MemoryUsage;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::shader::ShaderModule;
use vulkano::swapchain::Swapchain;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use crate::matrix_three::Matrix3;
use std::f32::consts::PI;
use std::sync::Arc;
use bevy_ecs::prelude::*;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

use super::branch_mesh_gen::BranchMesh;
use super::camera_maths::Camera;
use super::{
    general_graphics::{Normal, Vertex, get_generic_uniforms},
    gui::GUIData,
};

mod branch_vert_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "assets/shaders/branch_vert.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

mod branch_frag_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/branch_frag.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}


#[derive(Resource)]
pub struct BranchGraphicsResources {
    pub flat_shaded: bool,
    pub polygon_vectors: Vec<Vector3>,
}


/// Creates a polygon of vectors,
/// each of the vectors is a direction 
/// so can be used to generate a polygon from a central point. 
/// By default, the first direction will go along the x axis. 
/// A rotation will rotate by that many radians anticlockwise about the y axis
fn create_vector_polygon(sides: u32, rotation: Option<f32>) -> Vec<Vector3> {
    
    let inner_rotation_matrix = Matrix3::from_angle_y(2.0 * PI / sides as f32);

    let initial_direction = if rotation.is_none() {
        Vector3::X()
    } else {
        Vector3::X().transform(Matrix3::from_angle_y(-rotation.unwrap()))
    };

    let mut vectors = vec![initial_direction];
    for i in 0..(sides-1) as usize {
        vectors.push(vectors[i].clone().transform(inner_rotation_matrix));
    }

    vectors
}



//////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Gui ///////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////

/// creates the gui object for branch resources
pub fn add_world_branch_graphics_resources(
    world: &mut World,
) {
    world.insert_resource(BranchGraphicsResources {
        flat_shaded: false,
        polygon_vectors: create_vector_polygon(3, None)
    })
}

/// updates the branch resources from the gui
pub fn update_branch_resources(
    gui_query: Query<&GUIData>,
    mut branch_resources: ResMut<BranchGraphicsResources>,
) {
    for gui in gui_query.iter() {
        if gui.name == "branch graphics settings" {
            if gui.i32_sliders[0].1 != branch_resources.polygon_vectors.len() as i32 && gui.i32_sliders[0].1 > 2 {
                branch_resources.polygon_vectors = create_vector_polygon(gui.i32_sliders[0].1 as u32, None);
            }

            if gui.bools[0].1 != branch_resources.flat_shaded {
                // inverts a bool
                branch_resources.flat_shaded ^= true;
            }
        }
    }
}

/// creates the branch resource gui
pub fn create_branch_resources_gui(
    branch_resources: Res<BranchGraphicsResources>,
    mut commands: Commands,
) {
    commands.spawn(GUIData {
        name: "branch graphics settings".to_string(),
        bools: vec![("flat shade branches".to_string(), branch_resources.flat_shaded)],
        // where is 10 from you may ask? Well I'll tell you a secret, I pulled it out my ass
        i32_sliders: vec![("num branch vertices".to_string(), branch_resources.polygon_vectors.len() as i32, 3..=10)],
        ..Default::default()
    });
}



//////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Graphics //////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////

/// returns the branch vertex and fragment shaders
pub fn get_branch_shaders(
    device: &Arc<Device>,
) -> (Arc<ShaderModule>, Arc<ShaderModule>) {
    return (branch_vert_shader::load(device.clone()).unwrap(), branch_frag_shader::load(device.clone()).unwrap());
}

/// gets the branch meshes for one plant, effectively the plant mesh, using the same method as getting them from base to tip
fn get_branch_meshes(
    branch_meshes: &Query<&BranchMesh, With<BranchTag>>,
    connections: &Query<&BranchConnectionData, With<BranchTag>>,
    root_branch: Entity
) -> (Vec<Vertex>, Vec<Normal>, Vec<u32>){

    let mut vertices: Vec<Vertex> = vec![];
    let mut normals: Vec<Normal> = vec![];
    let mut indices: Vec<u32> = vec![];

    let branches = get_branches_base_to_tip(connections, root_branch);

    for id in branches.iter() {
        if let Ok(branch) = branch_meshes.get(*id) {
            let current_length = vertices.len() as u32;

            vertices.append(&mut branch.vertices.clone());
            normals.append(&mut branch.normals.clone());
            let mut new_indices = branch.indices.clone();
            new_indices.iter_mut().for_each(|x| *x += current_length);
            indices.append(&mut new_indices);
        }
    }

    (vertices, normals, indices)
}

/// gets the mesh data from all the plants
fn get_plants_mesh_data(
    plants: &Query<&PlantData, With<PlantTag>>,
    branch_meshes: &Query<&BranchMesh, With<BranchTag>>,
    branch_connections: &Query<&BranchConnectionData, With<BranchTag>>,
) -> (Vec<Vertex>, Vec<Normal>, Vec<u32>) {

    let mut vertices: Vec<Vertex> = vec![];
    let mut normals: Vec<Normal> = vec![];
    let mut indices: Vec<u32> = vec![];

    for plant in plants.iter() {
        if plant.root_node.is_none() {continue;}
        let (mut new_vertices, mut new_normals, mut new_indices) = get_branch_meshes(branch_meshes, branch_connections, plant.root_node.unwrap());
        let current_length = vertices.len() as u32;
        new_indices.iter_mut().for_each(|x| *x += current_length);

        vertices.append(&mut new_vertices);
        normals.append(&mut new_normals);
        indices.append(&mut new_indices);
    }

    (vertices, normals, indices)
}

/// flat shades a set of triangles
fn flat_shade(
    in_vertices: Vec<Vertex>,
    in_indices: Vec<u32>,
) -> (Vec<Vertex>, Vec<Normal>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut normals: Vec<Normal> = Vec::new();

    for i in (0..in_indices.len()-1).step_by(3) {
        let v_one: Vector3 = in_vertices[in_indices[i as usize + 0] as usize].into();
        let v_two: Vector3 = in_vertices[in_indices[i as usize + 1] as usize].into();
        let v_thr: Vector3 = in_vertices[in_indices[i as usize + 2] as usize].into();

        let normal = {
            let normal = (v_two - v_one).cross(&(v_thr - v_one));
            Normal{normal: normal.into()}
        };

        vertices.push(Vertex { position: v_one.into() });
        vertices.push(Vertex { position: v_two.into() });
        vertices.push(Vertex { position: v_thr.into() });
        normals.push(normal);
        normals.push(normal);
        normals.push(normal);
    }

    let indices = (0..(vertices.len() - 1) as u32).collect_vec();
    (vertices, normals, indices)
}

/// adds the commands to draw branches to the given builder
pub fn add_branch_draw_commands(
    builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
    graph_pipeline: &Arc<GraphicsPipeline>,
    descriptor_allocator: &StandardDescriptorSetAllocator,
    uniform_buffer_subbuffer: &Arc<CpuBufferPoolSubbuffer<branch_vert_shader::ty::Data>>,
    mem_allocator: &Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,

    world: &mut World,
) {

    // create the system state to query data and then query it
    let mut state: SystemState<(
        Query<&PlantData, With<PlantTag>>,
        Query<&BranchMesh, With<BranchTag>>,
        Query<&BranchConnectionData, With<BranchTag>>,
        Res<BranchGraphicsResources>,
    )> = SystemState::new(world);

    let (plants, branch_meshes, branch_connections, branch_graphics_res) = state.get(world);

    let (vertices, normals, indices) = {
        if branch_graphics_res.flat_shaded {
            let (vertices, _normals, indices) = get_plants_mesh_data(&plants, &branch_meshes, &branch_connections);
            flat_shade(vertices, indices)
        } else {
            get_plants_mesh_data(&plants, &branch_meshes, &branch_connections)
        }
    };

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        mem_allocator,
        BufferUsage{
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        vertices
    ).unwrap();

    let normal_buffer = CpuAccessibleBuffer::from_iter(
        mem_allocator,
        BufferUsage{
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        normals
    ).unwrap();

    let index_buffer = CpuAccessibleBuffer::from_iter(
        mem_allocator,
        BufferUsage{
            index_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        indices
    ).unwrap();

    let layout = graph_pipeline.layout().set_layouts().get(0).unwrap();
    let uniforms_set = PersistentDescriptorSet::new(
        descriptor_allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer.clone())],
    )
    .unwrap();

    builder
        .bind_pipeline_graphics(graph_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            graph_pipeline.layout().clone(),
            0,
            uniforms_set
        )
        .bind_vertex_buffers(0, (vertex_buffer, normal_buffer))
        .bind_index_buffer(index_buffer.clone())
        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
        .unwrap();
}


pub fn create_branch_uniform_buffer(
    memory_allocator: &Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>
) -> CpuBufferPool<branch_vert_shader::ty::Data> {
    CpuBufferPool::<branch_vert_shader::ty::Data>::new(
        memory_allocator.clone(),
        BufferUsage {
            uniform_buffer: true,
            ..BufferUsage::empty()
        },
        MemoryUsage::Upload
    )
}

pub fn update_branch_uniform_buffer(
    swapchain: &Arc<Swapchain>,
    camera: &Camera,
    branch_uniform_buffer: &CpuBufferPool<branch_vert_shader::ty::Data>,
) -> Arc<CpuBufferPoolSubbuffer<branch_vert_shader::ty::Data>> {
    let (view, proj) = get_generic_uniforms(swapchain, camera);
    let data = branch_vert_shader::ty::Data {
        view: view.into(),
        proj: proj.into()
    };
    branch_uniform_buffer.from_data(data).unwrap()
}
