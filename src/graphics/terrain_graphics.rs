use vulkano::{
    DeviceSize, NonExhaustive,
    device::{Device, Queue},
    buffer::{Buffer, BufferUsage, BufferCreateInfo, Subbuffer, allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo}},
    descriptor_set::{DescriptorSetWithOffsets, PersistentDescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator, layout::{DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType}},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferUsage, CopyBufferInfo, SecondaryAutoCommandBuffer, allocator::StandardCommandBufferAllocator, PrimaryAutoCommandBuffer},
    memory::allocator::{FreeListAllocator, AllocationCreateInfo, MemoryUsage, GenericMemoryAllocator},
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, graphics::{depth_stencil::DepthStencilState, input_assembly::InputAssemblyState, vertex_input::Vertex, viewport::*}, layout::PipelineLayoutCreateInfo},
    render_pass::{RenderPass, Subpass},
    shader::{DescriptorBindingRequirements, DescriptorRequirements, ShaderModule, ShaderStages},
    swapchain::Swapchain,
    sync::{self, GpuFuture}
};
use bevy_ecs::{prelude::*, system::SystemState};
use std::{sync::Arc, collections::BTreeMap};
use egui::epaint::ahash::HashMap;

use super::{
    mesh::Mesh,
    camera_maths::Camera,
    general_graphics::{Normal, PositionVertex, get_generic_uniforms, basic_frag_shader},
    gui::GUIData,
    super::{
        branch::*,
        maths::{vector_three::Vector3, matrix_three::Matrix3},
        terrain::TerrainTag,
    }
};


mod heightmap_vert_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "assets/shaders/heightmap_terrain_vert.glsl",
        include: ["assets/shaders/include/light_maths.glsl"]
    }
}


#[derive(Resource)]
pub struct TerrainMeshBuffers {
    pub vertices: Subbuffer<[PositionVertex]>,
    pub normals: Subbuffer<[Normal]>,
    pub indices: Subbuffer<[u32]>,
}

pub fn create_terrain_mesh_buffers(
    buffer_allocator: &Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    world: &mut World
) {

    let mut state: SystemState<(
        Query<&Mesh, With<TerrainTag>>,
    )> = SystemState::new(world);

    let meshes = state.get(world).0;

    let (vertices, normals, indices) = {
        let (vertices, normals, indices) = meshes.single().get_components();
        if vertices.len() == 0 {
            (vec![Vector3::ZERO().into(), Vector3::ZERO().into(), Vector3::ZERO().into()],
            vec![Vector3::Y().into(), Vector3::Y().into(), Vector3::Y().into()], 
            vec![0, 1, 2])
        } else {(vertices, normals, indices)}
    };
    

    let vertex_buffer = Buffer::from_iter(
        buffer_allocator,
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
        buffer_allocator,
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
        buffer_allocator,
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


    world.insert_resource(TerrainMeshBuffers{
        vertices: vertex_buffer,
        normals: normal_buffer,
        indices: index_buffer,
    })
}




fn get_terrain_graphics_pipeline_layout(
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


pub fn get_terrain_pipeline(
    dimensions: [u32; 2],
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> Arc<GraphicsPipeline>{
    let vs = heightmap_vert_shader::load(device.clone()).unwrap();
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
        .with_pipeline_layout(device.clone(), get_terrain_graphics_pipeline_layout(device))
        // .build(device.clone())
        .unwrap();
        

    pipeline
}


//////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// HeightMap Terrain /////////////////////////////
//////////////////////////////////////////////////////////////////////////////////

pub fn get_heightmap_light_buffers(
    point_lights: Vec<(Vector3, f32)>,
    directional_lights: Vec<(Vector3, f32)>,
    mem_allocator: &Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
) -> (Subbuffer<[heightmap_vert_shader::PointLight]>, Subbuffer<[heightmap_vert_shader::DirectionalLight]>) {

    let point_light_data = {
        let mut data: Vec<heightmap_vert_shader::PointLight> = Vec::new();
        for light in point_lights.iter() {
            data.push(heightmap_vert_shader::PointLight {position: light.0.into(), intensity: light.1});
        }
        if data.len() == 0 {
            data = vec![heightmap_vert_shader::PointLight {position: Vector3::ZERO().into(), intensity: 0.0}];
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
        let mut data: Vec<heightmap_vert_shader::DirectionalLight> = Vec::new();
        for light in directional_lights.iter() {
            data.push(heightmap_vert_shader::DirectionalLight {direction: (-light.0).normalised().into(), intensity: light.1});
        }
        if data.len() == 0 {
            data = vec![heightmap_vert_shader::DirectionalLight {direction: Vector3::ZERO().into(), intensity: 0.0}];
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

// adds the commands to draw heightmap terrain to the given builder
pub fn add_heightmap_terrain_draw_commands(
    builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
    graph_pipeline: &Arc<GraphicsPipeline>,
    descriptor_allocator: &StandardDescriptorSetAllocator,
    vert_uniform_buffer: &Subbuffer<heightmap_vert_shader::Data>,
    frag_light_buffers: &(Subbuffer<[heightmap_vert_shader::PointLight]>, Subbuffer<[heightmap_vert_shader::DirectionalLight]>),

    world: &mut World,
) {

    // create the system state to query data and then query it
    let mut state: SystemState<(
        Res<TerrainMeshBuffers>,
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



pub fn create_heightmap_uniform_buffer(
    swapchain: &Arc<Swapchain>,
    camera: &Camera,
    grass_colour: [f32; 3],
    rock_colour: [f32; 3],
    grass_slope_threshold: f32,
    grass_blend_amount: f32,
    terrain_uniform: &SubbufferAllocator,
) -> Subbuffer<heightmap_vert_shader::Data> {
    let (view, proj) = get_generic_uniforms(swapchain, camera);
    let data = heightmap_vert_shader::Data {
        view: view.into(),
        proj: proj.into(),
        grass_colour: grass_colour.into(),
        rock_colour: rock_colour.into(),
        grass_slope_threshold: grass_slope_threshold.into(),
        grass_blend_amount: grass_blend_amount.into(),
    };
    let subbuffer = terrain_uniform.allocate_sized().unwrap();
    *subbuffer.write().unwrap() = data;
    subbuffer
}




//////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Flat Terrain //////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////