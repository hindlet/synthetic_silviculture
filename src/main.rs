#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use vulkano::{
    pipeline::graphics::vertex_input::BuffersDefinition,
    sync::{self, GpuFuture, FlushError},
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    swapchain::{acquire_next_image, AcquireError, SwapchainPresentInfo},
    command_buffer::{AutoCommandBufferBuilder, RenderPassBeginInfo, CommandBufferUsage, SubpassContents}
};
use winit::{
    event::{Event, WindowEvent, ElementState},
    window::Window,
    event_loop::ControlFlow,
};

use image::*;
use rand::*;

mod branch;
use branch::*;

mod branch_prototypes;
use branch_prototypes::*;

mod plant;
use plant::*;

mod tests;

mod general;
use general::*;
use vector_three::Vector3;

mod graphics;
use graphics::{branch_mesh_gen::*, branch_graphics::*, general_graphics::*, camera_maths::*};

mod branch_node;
use branch_node::*;

mod transform;
use transform::*;

mod branch_development;


fn main() {
    let mut world = World::new();

    // add stuff to the world
    add_world_branch_graphics_resources(&mut world);


    // graphics
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup();
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = get_single_renderpass(&device, &swapchain);

    let mut camera = Camera{position: Vector3::X() * -10.0, ..Default::default()};

    let (vs, fs) = get_branch_shaders(&device);

    let buffers_defintion = BuffersDefinition::new()
        .vertex::<Vertex>()
        .vertex::<Normal>();

    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(&memory_allocator, &vs, &fs, &swapchain_images, &render_pass, buffers_defintion);

    // this determines if the swapchain needs to be rebuilt
    let mut recreate_swapchain = false;

    // i think this has info about thw previous frame, so we can execute from it
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    // memory allocators
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());

    // uniform buffers
    let branch_uniform_buffer = create_branch_uniform_buffer(&memory_allocator);

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

                    let buffers_defintion = BuffersDefinition::new()
                    .vertex::<Vertex>()
                    .vertex::<Normal>();

                    if let Ok((new_swapchain, new_pipeline, new_framebuffers)) = recreate_swapchain_and_pipeline(swapchain.clone(), dimensions, &memory_allocator, &vs, &fs, &render_pass, buffers_defintion) {
                        swapchain = new_swapchain;
                        pipeline = new_pipeline;
                        framebuffers = new_framebuffers;
                        recreate_swapchain = false
                    }
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
                let mut builder = AutoCommandBufferBuilder::primary(&command_buffer_allocator, queue.queue_family_index(), CommandBufferUsage::OneTimeSubmit).unwrap();
                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.2, 0.2, 0.2, 1.0].into()),
                                Some(1f32.into()),
                            ],
                            ..RenderPassBeginInfo::framebuffer(framebuffers[image_num as usize].clone())
                        },
                        SubpassContents::Inline,
                    )
                    .unwrap();

                let branch_uniforms = update_branch_uniform_buffer(&swapchain, &camera, &branch_uniform_buffer);
                    

                // add_branch_draw_commands(&mut builder, &pipeline, &descriptor_set_allocator, &branch_uniforms, &memory_allocator, &mut world);

                builder.end_render_pass().unwrap();
                let branch_commands = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), branch_commands)
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