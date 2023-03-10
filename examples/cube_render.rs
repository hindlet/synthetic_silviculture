use synthetic_silviculture::graphics::{test_cube, general_graphics::*, camera_maths::*};
use synthetic_silviculture::general::{matrix_three::Matrix3, matrix_four::Matrix4, vector_three::*};

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, CpuBufferPool, cpu_pool::CpuBufferPoolSubbuffer, TypedBufferAccess};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::swapchain::{SwapchainCreationError, SwapchainCreateInfo, acquire_next_image, AcquireError, Swapchain, SwapchainPresentInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, CommandBufferUsage, RenderPassBeginInfo, SubpassContents};
use vulkano::device::Queue;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer};
use vulkano::sync::{self, GpuFuture};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator};
use vulkano::sync::FlushError;
use vulkano::memory::allocator::MemoryUsage;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::ControlFlow,
    window::Window,
};

use std::time::Instant;
use std::sync::Arc;

// shader modules
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "examples/example_shaders/cube_render_vert.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};
            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "examples/example_shaders/cube_render_frag.glsl",
    }
}





fn main() {
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup();
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = get_renderpass(&device, &swapchain);
    

    // create data buffers
    let vertex_buffer = CpuAccessibleBuffer::from_iter (
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        test_cube::VERTICES,
    ).unwrap();

    let normals_buffer = CpuAccessibleBuffer::from_iter (
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        test_cube::NORMALS,
    ).unwrap();

    let index_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            index_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        test_cube::INDICES
    ).unwrap();

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(
        memory_allocator.clone(),
        BufferUsage {
            uniform_buffer: true,
            ..BufferUsage::empty()
        },
        MemoryUsage::Upload
    );

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    let buffers_defintion = BuffersDefinition::new()
        .vertex::<ColouredVertex>()
        .vertex::<Normal>();

    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(&memory_allocator, &vs, &fs, &swapchain_images, &render_pass, buffers_defintion);
    let mut camera = Camera {
        position: Vector3::new(3.0, 1.0, 3.0),
        move_speed: 0.1,
        ..Default::default()
    };
    camera.look_at(Vector3::new(0.0, 0.0, 0.0));

    println!("Camera_controls:\nW/S: forward/back\nA/D: left/right\nSpace/C: up/down\nQ/E: rotate right/left\nR/F: rotate up/down");

    // this determines if the swapchain needs to be rebuilt
    let mut recreate_swapchain = false;

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    let rotation_start = Instant::now(); // time used to calculate cube spin

    // run the loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            // close the window if needed
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => { *control_flow = ControlFlow::Exit; }

            // recreate the swapchain if window resized
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => { recreate_swapchain = true }

            // handle camera control
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input:winit::event::KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state,
                        ..
                    },
                    .. 
                },
                ..
            } => {
                camera.process_key(keycode, state == ElementState::Pressed);
            }

            Event::MainEventsCleared => {
                camera.do_move();
            }

            // do the actual stuff
            Event::RedrawEventsCleared => {

                // check theres actually a window to draw on
                let dimensions = surface.object().unwrap().downcast_ref::<Window>().unwrap().inner_size();
                if dimensions.width == 0 || dimensions.height == 0 {
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                // reacreate swapchain if nessesary
                if recreate_swapchain {
                    // get a new swapchain and images
                    let (new_swapchain, new_swapchain_images) = 
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                    swapchain = new_swapchain;
                    
                    // get a new pipeline and framebuffers
                    let buffers_defintion = BuffersDefinition::new()
                        .vertex::<ColouredVertex>()
                        .vertex::<Normal>();
                    let (new_pipeline, new_framebuffers) = window_size_dependent_setup(&memory_allocator, &vs, &fs, &new_swapchain_images, &render_pass, buffers_defintion);
                    pipeline = new_pipeline;
                    framebuffers = new_framebuffers;

                    recreate_swapchain = false
                }

                // get the image number and future for the next freame
                let (image_num, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                // create uniform and command buffers for the frame
                let uniform_buffer_subbuffer = get_uniform_subbuffer(&rotation_start, &swapchain, &camera, &uniform_buffer);
                let command_buffer = get_command_buffers(&descriptor_set_allocator, &command_buffer_allocator, &queue, &pipeline, &framebuffers, &vertex_buffer, &normals_buffer, &index_buffer, &uniform_buffer_subbuffer, image_num as usize);

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_num)
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }


            }
            _ => (),
        }
    })
}




fn get_uniform_subbuffer (
    rotation_start: &Instant,
    swapchain: &Arc<Swapchain>,
    camera: &Camera,
    uniform_buffer: &CpuBufferPool<vs::ty::Data>,
) -> Arc<CpuBufferPoolSubbuffer<vs::ty::Data>> {

    let elapsed = rotation_start.elapsed();
    let rotation = 
        elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(rotation);

    let (view, proj) = get_generic_uniforms(swapchain, camera);
    

    let uniform_data = vs::ty::Data {
        world: Matrix4::from(rotation).into(),
        view: view.into(),
        proj: proj.into(),
    };

    uniform_buffer.from_data(uniform_data).unwrap()

}


fn get_command_buffers(
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Arc<CpuAccessibleBuffer<[ColouredVertex]>>,
    normal_buffer: &Arc<CpuAccessibleBuffer<[Normal]>>,
    index_buffer: &Arc<CpuAccessibleBuffer<[u16]>>,
    uniform_buffer_subbuffer: &Arc<CpuBufferPoolSubbuffer<vs::ty::Data>>,
    image_num: usize
) -> PrimaryAutoCommandBuffer {

    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    let set = PersistentDescriptorSet::new(
        descriptor_set_allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer.clone())],
    )
    .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([0.2, 0.2, 0.2, 1.0].into()),
                    Some(1f32.into()),
                ],
                ..RenderPassBeginInfo::framebuffer(framebuffers[image_num].clone())
            },
            SubpassContents::Inline,
        )
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipeline.layout().clone(),
            0,
            set,
        )
        .bind_vertex_buffers(0, (vertex_buffer.clone(), normal_buffer.clone()))
        .bind_index_buffer(index_buffer.clone())
        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap();
    builder.build().unwrap()
}