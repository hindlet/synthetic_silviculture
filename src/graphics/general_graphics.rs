#![allow(unused_variables, unused_imports)]

use std::sync::Arc;
use bevy_ecs::prelude::*;
use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateInfo},
    device::{Device, DeviceExtensions, DeviceCreateInfo, physical::{PhysicalDevice, PhysicalDeviceType}, Queue, QueueCreateInfo, DeviceOwned},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    shader::ShaderModule,
    render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass},
    image::{SwapchainImage, ImageAccess, view::ImageView, AttachmentImage, ImageUsage, swapchain},
    format::Format,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint,
    graphics::{depth_stencil::DepthStencilState, viewport::{Viewport, ViewportState}, input_assembly::InputAssemblyState, vertex_input::BuffersDefinition}
    },
    impl_vertex, buffer::BufferContents,
    buffer::{CpuAccessibleBuffer, cpu_pool::CpuBufferPoolSubbuffer, TypedBufferAccess},
    command_buffer::{PrimaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents},
    memory::allocator::StandardMemoryAllocator,
};
use vulkano_win::VkSurfaceBuild;
use winit::{window::{WindowBuilder, Window}, event::VirtualKeyCode};
use winit::event_loop::EventLoop;
use bytemuck::{Pod, Zeroable};
use crate::{graphics::camera_maths::Camera, general::vector_three::Vector3};
use crate::general::matrix_four::Matrix4;
use crate::general::matrix_three::Matrix3;
use crate::graphics::gui::{GUIResources, create_gui_subpass};




// define Vertex and Normal Structs
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct ColouredVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
impl_vertex!(ColouredVertex, position, color);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
}
impl_vertex!(Vertex, position);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Normal {
    pub normal: [f32; 3],
}
impl_vertex!(Normal, normal);





// returns the basic things needed for graphics processing
pub fn base_graphics_setup() -> (
    Arc<Queue>,
    Arc<Device>,
    Arc<PhysicalDevice>,
    Arc<Surface>,
    EventLoop<()>,
    Arc<StandardMemoryAllocator>,
) {
    // get the library and create a vulkan instance with required extensions
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
        library, 
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enumerate_portability: true, // this is so it works on mac
            ..Default::default()
        },
    ).expect("failed to create instance");

    // setup window and event loop
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();


    // get device
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = get_physical_device(&instance, &surface, &device_extensions);

    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            // here we pass the desired queue family to use by index
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create device");


    let queue = queues.next().unwrap();

    let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    (queue, device, physical_device, surface, event_loop, allocator)
}

// loops through all devices and finds one with required extensions if it exists
fn get_physical_device (
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {

    instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                // find the first suitable queue family
                // if there isn't one, None is returned and device is disqualified
                .position(|(i, q)| {
                    q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4
        })
        .expect("no device available")
}


pub fn get_swapchain(
    physical_device: &Arc<PhysicalDevice>,
    surface: &Arc<Surface>,
    device: &Arc<Device>
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>){
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = surface.object().unwrap().downcast_ref::<Window>().unwrap().inner_size();
    let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    let image_format = Some(
        physical_device
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );

    let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let image_format = Some(
            device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage {
                color_attachment: true,
                ..ImageUsage::empty()
            },
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap(),
            ..Default::default()
        },
    )
    .unwrap()
}


pub fn get_renderpass (
    device: &Arc<Device>,
    swapchain: &Arc<Swapchain>,
) -> Arc<RenderPass> {

    vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: Format::D16_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    ).unwrap()
}


pub fn window_size_dependent_setup(
    memory_allocator: &StandardMemoryAllocator,
    vs: &ShaderModule,
    fs: &ShaderModule,
    images: &[Arc<SwapchainImage>],
    render_pass: &Arc<RenderPass>,
    buffers_def: BuffersDefinition,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let dimensions = images[0].dimensions().width_height();

    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(memory_allocator, dimensions, Format::D16_UNORM).unwrap(),
    )
    .unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();


    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(
            buffers_def
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
        .build(memory_allocator.device().clone())
        .unwrap();

    (pipeline, framebuffers)
}


pub fn get_generic_uniforms(
    swapchain: &Arc<Swapchain>,
    camera: &Camera,
) -> (Matrix4, Matrix4){
    
    let aspect_ratio = swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;

    let proj = Matrix4::persective_matrix(
        std::f32::consts::FRAC_PI_2,
        aspect_ratio,
        0.01,
        100.0,
    );

    let scale = Matrix4::from(Matrix3::from_scale(1.0));

    (scale * camera.get_view_matrix(), proj)
}

