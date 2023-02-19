#![allow(unused_variables, unused_imports)]

use std::sync::Arc;

use cgmath::Matrix4;
use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateInfo},
    device::{Device, DeviceExtensions, DeviceCreateInfo, physical::{PhysicalDevice, PhysicalDeviceType}, Queue, QueueCreateInfo},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    shader::ShaderModule,
    render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass},
    image::{SwapchainImage, ImageAccess, view::ImageView, AttachmentImage, ImageUsage},
    format::Format,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint,
    graphics::{depth_stencil::DepthStencilState, viewport::{Viewport, ViewportState}, input_assembly::InputAssemblyState, vertex_input::BuffersDefinition}
    },
    impl_vertex, buffer::BufferContents,
    buffer::{CpuAccessibleBuffer, cpu_pool::CpuBufferPoolSubbuffer, TypedBufferAccess},
    command_buffer::{PrimaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents},
};
use vulkano_win::VkSurfaceBuild;
use winit::window::{WindowBuilder, Window};
use winit::event_loop::EventLoop;
use bytemuck::{Pod, Zeroable};
use crate::graphics::camera_maths::Camera;




// define Vertex and Normal Structs
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
pub fn base_setup() -> (
    Arc<Queue>,
    Arc<Device>,
    Arc<PhysicalDevice>,
    Arc<Surface<Window>>,
    EventLoop<()>,
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

    (queue, device, physical_device, surface, event_loop)
}

// loops through all devices and finds one with required extensions if it exists
fn get_physical_device (
    instance: &Arc<Instance>,
    surface: &Arc<Surface<Window>>,
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
    surface: &Arc<Surface<Window>>,
    device: &Arc<Device>
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>){
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = surface.window().inner_size();
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
            image_extent: surface.window().inner_size().into(),
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
    swapchain: &Arc<Swapchain<Window>>,
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
    device: &Arc<Device>,
    vs: &ShaderModule,
    fs: &ShaderModule,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: &Arc<RenderPass>,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let dimensions = images[0].dimensions().width_height();

    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap(),
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
            BuffersDefinition::new()
                .vertex::<Vertex>()
                .vertex::<Normal>()
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
        .build(device.clone())
        .unwrap();

    (pipeline, framebuffers)
}


pub fn get_generic_uniforms(
    swapchain: &Arc<Swapchain<Window>>,
    camera: &Camera,
) -> (Matrix4<f32>, Matrix4<f32>){
    
    let aspect_ratio = swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;

    let proj = cgmath::perspective(
        cgmath::Rad(std::f32::consts::FRAC_PI_2),
        aspect_ratio,
        0.01,
        100.0,
    );

    let scale = Matrix4::from_scale(1.0);

    (scale * camera.get_view_matrix(), proj)
}





// was trying something, it didn't work but when i wanna try again it's here


// #[macro_export]
// macro_rules! build_command_buffer_creation_function {

//     ($name:tt, $uniform_buffer_type:ty) => {
//         use std::sync::Arc;
//         use vulkano::{
//             device::{Device, Queue},
//             pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint},
//             render_pass::Framebuffer,
//             buffer::{CpuAccessibleBuffer, cpu_pool::CpuBufferPoolSubbuffer, TypedBufferAccess},
//             command_buffer::{PrimaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents},
//             memory::pool::StandardMemoryPool,
//             descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
//         };

//         fn $name(
//             device: &Arc<Device>,
//             queue: &Arc<Queue>,
//             pipeline: &Arc<GraphicsPipeline>,
//             framebuffers: &Vec<Arc<Framebuffer>>,
//             vertex_buffer: &Arc<CpuAccessibleBuffer<[Vertex]>>,
//             uniform_buffer_subbuffer: &Arc<CpuBufferPoolSubbuffer<&$uniform_buffer_type, Arc<StandardMemoryPool>>>,
//             image_num: usize
//         ) -> PrimaryAutoCommandBuffer{

//             let layout = pipeline.layout().set_layouts().get(0).unwrap();
//             let set = PersistentDescriptorSet::new(
//                 layout.clone(),
//                 [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer.clone())],
//             )
//             .unwrap();

//             let mut builder = AutoCommandBufferBuilder::primary(
//                 device.clone(),
//                 queue.queue_family_index(),
//                 CommandBufferUsage::OneTimeSubmit,
//             )
//             .unwrap();
//             builder
//                 .begin_render_pass(
//                     RenderPassBeginInfo {
//                         clear_values: vec![
//                             Some([0.0, 0.0, 1.0, 1.0].into()),
//                             Some(1f32.into()),
//                         ],
//                         ..RenderPassBeginInfo::framebuffer(framebuffers[image_num].clone())
//                     },
//                     SubpassContents::Inline,
//                 )
//                 .unwrap()
//                 .bind_pipeline_graphics(pipeline.clone())
//                 .bind_descriptor_sets(
//                     PipelineBindPoint::Graphics,
//                     pipeline.layout().clone(),
//                     0,
//                     set,
//                 )
//                 .bind_vertex_buffers(0, vertex_buffer.clone())
//                 .draw(vertex_buffer.len() as u32, 1, 0, 0)
//                 .unwrap()
//                 .end_render_pass()
//                 .unwrap();
//             builder.build().unwrap()
//         }
//     };
// }