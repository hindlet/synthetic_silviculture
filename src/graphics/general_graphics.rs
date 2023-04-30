#![allow(unused_variables, unused_imports)]

use std::sync::Arc;
use bevy_ecs::prelude::*;
use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateInfo},
    device::{Device, DeviceExtensions, DeviceCreateInfo, physical::{PhysicalDevice, PhysicalDeviceType}, Queue, QueueCreateInfo, DeviceOwned, QueueFlags},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError},
    shader::ShaderModule,
    render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass},
    image::{SwapchainImage, ImageAccess, view::ImageView, AttachmentImage, ImageUsage, swapchain},
    format::Format,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint,
    graphics::{depth_stencil::DepthStencilState, viewport::{Viewport, ViewportState}, input_assembly::InputAssemblyState, vertex_input::Vertex}, PipelineLayout, layout::PipelineLayoutCreateInfo
    },
    buffer::{BufferContents, BufferUsage, Buffer, BufferCreateInfo, Subbuffer},
    command_buffer::{PrimaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents},
    memory::allocator::{StandardMemoryAllocator, GenericMemoryAllocator, FreeListAllocator, MemoryUsage, AllocationCreateInfo},
};
use vulkano_win::VkSurfaceBuild;
use winit::{window::{WindowBuilder, Window}, event::VirtualKeyCode, dpi::PhysicalSize};
use winit::event_loop::EventLoop;
use bytemuck::{Pod, Zeroable};
use crate::{graphics::camera_maths::Camera, maths::vector_three::Vector3};
use crate::maths::matrix_four::Matrix4;
use crate::maths::matrix_three::Matrix3;
use crate::graphics::gui::{create_gui_subpass};






// define Vertex and Normal Structs
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, Vertex)]
pub struct ColouredVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, Vertex)]
pub struct PositionVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

impl Into<Vector3> for PositionVertex {
    fn into(self) -> Vector3 {
        self.position.into()
    }
}

impl From<Vector3> for PositionVertex {
    fn from(value: Vector3) -> Self {
        PositionVertex {position: value.into()}
    }
}

impl From<[f32; 3]> for PositionVertex {
    fn from(value: [f32; 3]) -> Self {
        PositionVertex {position: value}
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, Vertex)]
pub struct Normal {
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

impl From<Vector3> for Normal {
    fn from(value: Vector3) -> Self {
        Normal {normal: value.into()}
    }
}





// returns the basic things needed for graphics processing
pub fn base_graphics_setup(title: String) -> (
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
        .with_title(title)
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
                    q.queue_flags.intersects(QueueFlags::GRAPHICS) && p.surface_support(i as u32, &surface).unwrap_or(false)
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
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();
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
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .unwrap(),
            ..Default::default()
        },
    )
    .unwrap()
}

pub fn recreate_swapchain_and_framebuffers(
    old_swapchain: Arc<Swapchain>,
    dimensions: PhysicalSize<u32>,
    memory_allocator: &StandardMemoryAllocator,
    render_pass: &Arc<RenderPass>,
) -> Result<(Arc<Swapchain>, Vec<Arc<Framebuffer>>, [u32; 2]), ()> {

    let (new_swapchain, new_swapchain_images) = 
        match old_swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions.into(),
            ..old_swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return Err(()),
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
    
    // get a new pipeline and framebuffers
    let (new_framebuffers, new_dimensions) = get_framebuffers(memory_allocator, &new_swapchain_images, render_pass);
    Ok((new_swapchain, new_framebuffers, new_dimensions))
}


pub fn get_single_renderpass (
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


pub fn get_framebuffers(
    memory_allocator: &StandardMemoryAllocator,
    images: &[Arc<SwapchainImage>],
    render_pass: &Arc<RenderPass>
) -> (Vec<Arc<Framebuffer>>, [u32; 2]) {
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

    (framebuffers, dimensions)
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





pub mod basic_frag_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "assets/shaders/basic_frag.glsl",
    }
}

pub fn get_basic_frag_shader(
    device: &Arc<Device>
) -> Arc<ShaderModule> {
    basic_frag_shader::load(device.clone()).unwrap()
}
