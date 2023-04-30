use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use crate::{
    branch::*,
    maths::{vector_three::Vector3, matrix_three::Matrix3},
};
use super::{
    mesh::Mesh,
    camera_maths::Camera,
    general_graphics::{Normal, PositionVertex, get_generic_uniforms, basic_frag_shader},
    gui::GUIData,
};
use vulkano::{
    DeviceSize,
    NonExhaustive,
    buffer::{Buffer, BufferUsage, BufferCreateInfo, Subbuffer, allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo}},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferUsage, CopyBufferInfo, SecondaryAutoCommandBuffer, allocator::StandardCommandBufferAllocator, PrimaryAutoCommandBuffer},
    descriptor_set::{DescriptorSetWithOffsets, PersistentDescriptorSet, WriteDescriptorSet, allocator::{StandardDescriptorSetAllocator}, layout::{DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, DescriptorSetLayout}},
    device::{Device, Queue},
    memory::allocator::{FreeListAllocator, AllocationCreateInfo, MemoryUsage, GenericMemoryAllocator},
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, layout::PipelineLayoutCreateInfo, graphics::{depth_stencil::DepthStencilState, input_assembly::InputAssemblyState, vertex_input::Vertex, viewport::*}},
    render_pass::{RenderPass, Subpass},
    shader::{DescriptorBindingRequirements, DescriptorRequirements, ShaderModule, ShaderStages},
    swapchain::Swapchain,
    sync::{self, GpuFuture}
};
use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::sync::Arc;
use egui::epaint::ahash::HashMap;
use itertools::Itertools;


mod branch_vert_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "assets/shaders/branch_vert.glsl",
        include: ["assets/shaders/include/light_maths.glsl"]
    }
}


#[derive(Resource)]
pub struct BranchGraphicsResources {
    pub flat_shaded: bool,
    pub polygon_vectors: Vec<Vector3>,
    pub mem_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
}

#[derive(Resource)]
pub struct BranchMeshBuffers {
    pub vertices: Subbuffer<[PositionVertex]>,
    pub normals: Subbuffer<[Normal]>,
    pub indices: Subbuffer<[u32]>,
}

pub fn init_mesh_buffers_res(
    branch_mesh_query: Query<&Mesh, With<BranchTag>>,

    mesh_gen_res: Res<BranchGraphicsResources>,

    mut commands: Commands
) {
    let (vertices, normals, indices) = {
        let (vertices, normals, indices) = get_total_branch_mesh_data(&branch_mesh_query);
        if vertices.len() == 0 {
            (vec![Vector3::ZERO().into(), Vector3::ZERO().into(), Vector3::ZERO().into()],
            vec![Vector3::Y().into(), Vector3::Y().into(), Vector3::Y().into()], 
            vec![0, 1, 2])
        }
        else if mesh_gen_res.flat_shaded {
            Mesh::flat_shade_components(vertices, indices)
        } else {
            (vertices, normals, indices)
        }
    };

    


    let vertex_buffer = Buffer::from_iter(
        &mesh_gen_res.mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        vertices
    ).unwrap();

    let normal_buffer = Buffer::from_iter(
        &mesh_gen_res.mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        normals
    ).unwrap();

    let index_buffer = Buffer::from_iter(
        &mesh_gen_res.mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        indices
    ).unwrap();

    commands.insert_resource(BranchMeshBuffers {
        vertices: vertex_buffer,
        normals: normal_buffer,
        indices: index_buffer,
    })
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
    allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
) {
    world.insert_resource(BranchGraphicsResources {
        flat_shaded: false,
        polygon_vectors: create_vector_polygon(3, None),
        mem_allocator: allocator
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
pub fn get_branch_shader(
    device: &Arc<Device>,
) -> Arc<ShaderModule> {
    return branch_vert_shader::load(device.clone()).unwrap();
}

/// combines and returns all the branch meshes in the world
fn get_total_branch_mesh_data(
    branch_meshes: &Query<&Mesh, With<BranchTag>>,
) -> (Vec<PositionVertex>, Vec<Normal>, Vec<u32>){
    let mut vertices: Vec<PositionVertex> = vec![];
    let mut normals: Vec<Normal> = vec![];
    let mut indices: Vec<u32> = vec![];

    for mesh in branch_meshes.iter() {
        let current_length = vertices.len() as u32;

        vertices.append(&mut mesh.vertices.clone());
        normals.append(&mut mesh.normals.clone());
        let mut new_indices = mesh.indices.clone();
        new_indices.iter_mut().for_each(|x| *x += current_length);
        indices.append(&mut new_indices);
    }

    (vertices, normals, indices)
}



fn get_branch_graphics_pipeline_layout(
    device: &Arc<Device>,
) -> Arc<PipelineLayout> {


    let mut descriptor_requirements = {

        let mut map_one = HashMap::default();
        map_one.insert(Some(0_u32), DescriptorRequirements{
            memory_read: ShaderStages::VERTEX,
            ..Default::default()
        });

        let mut map_two = HashMap::default();
        map_two.insert(Some(0_u32), DescriptorRequirements{
            memory_read: ShaderStages::VERTEX,
            ..Default::default()
        });

        let mut map_three = HashMap::default();
        map_three.insert(Some(0_u32), DescriptorRequirements{
            memory_read: ShaderStages::VERTEX,
            ..Default::default()
        });

        vec![map_one, map_two, map_three]
    };

    let binding_requirements = vec![
        DescriptorBindingRequirements {
            descriptor_types: vec![DescriptorType::UniformBuffer],
            stages: ShaderStages::VERTEX,
            descriptors: descriptor_requirements.remove(0),
            descriptor_count: Some(1),
            ..Default::default()
        },
        DescriptorBindingRequirements {
            descriptor_types: vec![DescriptorType::StorageBuffer],
            stages: ShaderStages::VERTEX,
            descriptors: descriptor_requirements.remove(0),
            descriptor_count: Some(1),
            ..Default::default()
        },  
        DescriptorBindingRequirements {
            descriptor_types: vec![DescriptorType::StorageBuffer],
            stages: ShaderStages::VERTEX,
            descriptors: descriptor_requirements.remove(0),
            descriptor_count: Some(1),
            ..Default::default()
        }
    ];
    
    let mut buffer_descriptors: BTreeMap<u32, DescriptorSetLayoutBinding> = BTreeMap::default();
    buffer_descriptors.insert(0, DescriptorSetLayoutBinding::from(&binding_requirements[0]));
    buffer_descriptors.insert(1, DescriptorSetLayoutBinding::from(&binding_requirements[1]));
    buffer_descriptors.insert(2, DescriptorSetLayoutBinding::from(&binding_requirements[2]));


    let descriptor_set_layout = DescriptorSetLayout::new(
        device.clone(),
        DescriptorSetLayoutCreateInfo {
            bindings: buffer_descriptors,
            ..Default::default()
        }
    ).unwrap();


    PipelineLayout::new(
        device.clone(),
        PipelineLayoutCreateInfo {
            set_layouts: vec![descriptor_set_layout],
            ..Default::default()
        }
    ).unwrap()
}


pub fn get_branch_pipeline(
    dimensions: [u32; 2],
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let vs = branch_vert_shader::load(device.clone()).unwrap();
    let fs = basic_frag_shader::load(device.clone()).unwrap();

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(
            [PositionVertex::per_vertex(), Normal::per_vertex()]
        )
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .with_pipeline_layout(device.clone(), get_branch_graphics_pipeline_layout(device))
        // .build(device.clone())
        .unwrap();
        

    pipeline
}

pub fn get_branch_light_buffers(
    point_lights: Vec<(Vector3, f32)>,
    directional_lights: Vec<(Vector3, f32)>,
    mem_allocator: &Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
) -> (Subbuffer<[branch_vert_shader::PointLight]>, Subbuffer<[branch_vert_shader::DirectionalLight]>) {

    let point_light_data = {
        let mut data: Vec<branch_vert_shader::PointLight> = Vec::new();
        for light in point_lights.iter() {
            data.push(branch_vert_shader::PointLight {position: light.0.into(), intensity: light.1});
        }
        if data.len() == 0 {
            data = vec![branch_vert_shader::PointLight {position: Vector3::ZERO().into(), intensity: 0.0}];
        }
        data
    };

    let point_buffer = Buffer::from_iter(
        mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        point_light_data
    ).unwrap();

    let dir_light_data = {
        let mut data: Vec<branch_vert_shader::DirectionalLight> = Vec::new();
        for light in directional_lights.iter() {
            data.push(branch_vert_shader::DirectionalLight {direction: light.0.normalised().into(), intensity: light.1});
        }
        if data.len() == 0 {
            data = vec![branch_vert_shader::DirectionalLight {direction: Vector3::ZERO().into(), intensity: 0.0}];
        }
        data
    };

    let directional_buffer = Buffer::from_iter(
        mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        dir_light_data
    ).unwrap();

    (point_buffer, directional_buffer)
}


pub fn update_branch_data_buffers(
    branch_mesh_query: Query<&Mesh, With<BranchTag>>,

    mesh_gen_res: Res<BranchGraphicsResources>,
    mut buffers_res: ResMut<BranchMeshBuffers>,
) {

    let (vertices, normals, indices) = {
        let (vertices, normals, indices) = get_total_branch_mesh_data(&branch_mesh_query);
        if vertices.len() == 0 {
            (vec![Vector3::ZERO().into(), Vector3::ZERO().into(), Vector3::ZERO().into()],
            vec![Vector3::Y().into(), Vector3::Y().into(), Vector3::Y().into()], 
            vec![0, 1, 2])
        }
        else if mesh_gen_res.flat_shaded {
            Mesh::flat_shade_components(vertices, indices)
        } else {
            (vertices, normals, indices)
        }
    };


    let vertex_buffer = Buffer::from_iter(
        &mesh_gen_res.mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        vertices
    ).unwrap();

    let normal_buffer = Buffer::from_iter(
        &mesh_gen_res.mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        normals
    ).unwrap();

    let index_buffer = Buffer::from_iter(
        &mesh_gen_res.mem_allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        indices
    ).unwrap();

    buffers_res.vertices = vertex_buffer;
    buffers_res.normals = normal_buffer;
    buffers_res.indices = index_buffer;
}

/// adds the commands to draw branches to the given builder
pub fn add_branch_draw_commands(
    builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
    graph_pipeline: &Arc<GraphicsPipeline>,
    descriptor_allocator: &StandardDescriptorSetAllocator,
    vert_uniform_buffer: &Subbuffer<branch_vert_shader::Data>,
    frag_light_buffers: &(Subbuffer<[branch_vert_shader::PointLight]>, Subbuffer<[branch_vert_shader::DirectionalLight]>),

    world: &mut World,
) {

    // create the system state to query data and then query it
    let mut state: SystemState<(
        Res<BranchMeshBuffers>,
    )> = SystemState::new(world);

    let buffers = state.get(world).0;


    let layout = graph_pipeline.layout().set_layouts().get(0).unwrap();
    let uniforms_set = PersistentDescriptorSet::new(
        descriptor_allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, vert_uniform_buffer.clone()), WriteDescriptorSet::buffer(1, frag_light_buffers.0.clone()), WriteDescriptorSet::buffer(2, frag_light_buffers.1.clone())],
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
        .bind_vertex_buffers(0, (buffers.vertices.clone(), buffers.normals.clone()))
        .bind_index_buffer(buffers.indices.clone())
        .draw_indexed(buffers.indices.len() as u32, 1, 0, 0, 0)
        .unwrap();
}


pub fn create_branch_uniform_buffer(
    memory_allocator: &Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>
) -> SubbufferAllocator {
    SubbufferAllocator::new(
        memory_allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        }
    )
}

pub fn update_branch_uniform_buffer(
    swapchain: &Arc<Swapchain>,
    camera: &Camera,
    branch_uniform_buffer: &SubbufferAllocator,
) -> Subbuffer<branch_vert_shader::Data> {
    let (view, proj) = get_generic_uniforms(swapchain, camera);
    let data = branch_vert_shader::Data {
        view: view.into(),
        proj: proj.into()
    };
    let subbuffer = branch_uniform_buffer.allocate_sized().unwrap();
    *subbuffer.write().unwrap() = data;
    subbuffer
}
