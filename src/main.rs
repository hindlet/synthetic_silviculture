#![allow(dead_code, unused_variables, unused_imports)]
mod branch;
mod plant;
mod graphics;
mod general;
mod branch_node;
mod branch_prototypes;
mod transform;
mod branch_development;
mod environment;
mod fixed_schedule;

use std::sync::Arc;

use bevy_ecs::prelude::*;
use vulkano::{
    pipeline::graphics::vertex_input::BuffersDefinition,
    sync::{self, GpuFuture, FlushError},
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    swapchain::{acquire_next_image, AcquireError, SwapchainPresentInfo, Swapchain},
    command_buffer::{AutoCommandBufferBuilder, RenderPassBeginInfo, CommandBufferUsage, SubpassContents, CommandBufferInheritanceInfo}, render_pass::{Subpass, RenderPass}, device::Device, format::Format
};
use winit::{
    event::{Event, WindowEvent, ElementState},
    window::Window,
    event_loop::ControlFlow,
};
use image::*;
use rand::*;
use branch::*;
use branch_prototypes::*;
use plant::*;
use general::*;
use vector_three::Vector3;
use graphics::{branch_mesh_gen::*, branch_graphics::*, general_graphics::*, camera_maths::*, gui::*};
use branch_node::*;
use transform::*;




fn double_sample_render_pass(
    device: &Arc<Device>,
    swapchain: &Arc<Swapchain>,
) -> Arc<RenderPass> {

    vulkano::ordered_passes_renderpass!(device.clone(),
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
        passes: [
                { color: [color], depth_stencil: {depth}, input: [] }, // Draw what you want on this pass
                { color: [color], depth_stencil: {depth}, input: [] } // Gui render pass
            ]
    ).unwrap()
}

fn main() {
    let mut world = World::new();


    // do all the shader stuff
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup("synthetic silviculture".to_string());
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = double_sample_render_pass(&device, &swapchain);
    let branch_subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    let gui_subpass = Subpass::from(render_pass.clone(), 1).unwrap();

    world.spawn(MeshUpdateQueue {
        0: Vec::new()
    });


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

    // gui
    let gui = create_gui_from_subpass(&event_loop, &surface, &queue, &gui_subpass);
    add_world_gui_resources(&mut world, gui);
    add_world_branch_graphics_resources(&mut world);

    // scheduling
    
    let mut startup_schedule = Schedule::default();
    startup_schedule.add_systems((update_next_mesh, create_branch_resources_gui));

    let mut gui_schedule = Schedule::default();
    gui_schedule.add_systems((draw_gui_objects, update_branch_resources.after(draw_gui_objects)));

    let mut update_schedule = Schedule::default();
    update_schedule.add_system(check_for_force_update);

    // finally actual simulation stuff

    
    startup_schedule.run(&mut world);

    // uniforms
    let branch_uniform_buffer = create_branch_uniform_buffer(&memory_allocator);

    event_loop.run(move |event, _, control_flow| {
        match event {
            
            Event::WindowEvent { window_id: _, event } => {
                // pass things to gui
                let _pass_events_to_game = !pass_winit_event_to_gui(world.get_resource_mut::<GUIResources>(), &event);
                // check for resize or close
                match event {
                    WindowEvent::Resized(_) => {
                        recreate_swapchain = true
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput {
                        input:winit::event::KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state,
                            ..
                        },
                        .. 
                    } => {
                        camera.process_key(keycode, state == ElementState::Pressed);
                    }
                    _ => (),
                }
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

                // gui
                gui_schedule.run(&mut world);
                // update
                update_schedule.run(&mut world);

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

                // branch commands
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
                        SubpassContents::SecondaryCommandBuffers,
                    )
                    .unwrap();

                let mut secondary_builder = AutoCommandBufferBuilder::secondary(
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                    CommandBufferInheritanceInfo {
                        render_pass: Some(branch_subpass.clone().into()),
                        ..Default::default()
                    },
                ).unwrap();
                    
                let branch_uniforms = update_branch_uniform_buffer(&swapchain, &camera, &branch_uniform_buffer);
                add_branch_draw_commands(&mut secondary_builder, &pipeline, &descriptor_set_allocator, &branch_uniforms, &memory_allocator, &mut world);
                
                builder.execute_commands(secondary_builder.build().unwrap()).unwrap();
                builder.next_subpass(SubpassContents::SecondaryCommandBuffers).unwrap();

                // gui commands
            
                let gui_command_buffer = get_gui_resource_commands(world.get_resource_mut::<GUIResources>(), dimensions.into());
                builder.execute_commands(gui_command_buffer).unwrap().end_render_pass().unwrap();
                let draw_commands = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), draw_commands)
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