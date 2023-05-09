use std::sync::Arc;
use synthetic_silviculture::{
    terrain::spawn_heightmap_terrain,
    graphics::{
        camera_maths::Camera,
        terrain_graphics::*,
        general_graphics::*
    },
};
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents, CommandBufferInheritanceInfo, allocator::StandardCommandBufferAllocator},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Device,
    format::Format,
    render_pass::{RenderPass, Subpass},
    swapchain::{acquire_next_image, SwapchainPresentInfo, AcquireError, Swapchain},
    sync::{self, GpuFuture, FlushError},
};
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::ControlFlow,
    window::Window,
};
use bevy_ecs::prelude::*;


fn main() {
    let mut world = World::new();

    const GRASS_COLOUR: [f32; 3] = [0.0, 0.604, 0.090];
    const ROCK_COLOUR: [f32; 3] = [0.502, 0.518, 0.529];
    // these two need to be in range 0->1
    const GRASS_SLOPE_THRESHOLD: f32 = 0.1;
    const GRASS_BLEND_AMOUNT: f32 = 1.0;


    // do all the shader stuff
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup("terrain_render_example".to_string());
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = single_pass_renderpass(&device, &swapchain);
    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();


    let mut camera = Camera::new(Some([-4.0, 3.0, 0.0]), None, None, None);

    let (mut framebuffers, window_dimensions) = get_framebuffers(&memory_allocator, &swapchain_images, &render_pass);
    let mut pipeline = get_terrain_pipeline(window_dimensions, &device, &render_pass);


    // this determines if the swapchain needs to be rebuilt
    let mut recreate_swapchain = false;

    // i think this has info about thw previous frame, so we can execute from it
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    // memory allocators
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());


    // scheduling
    spawn_heightmap_terrain(100.0, 50, 10.0, [0, 0, 0], "assets/Noise_Texture.png".into(), &mut world);
    create_terrain_mesh_buffers(&memory_allocator, &mut world);

    // uniforms
    let uniform_allocator = create_uniform_buffer_allocator(&memory_allocator);
    let lighting_uniforms = get_heightmap_light_buffers(Vec::new(), vec![([0.7, -0.5, 0.2], 1.0)], &memory_allocator);

    event_loop.run(move |event, _, control_flow| {
        match event {
            
            Event::WindowEvent { window_id: _, event } => {
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

                    if let Ok((new_swapchain, new_framebuffers, new_dimensions)) = recreate_swapchain_and_framebuffers(swapchain.clone(), dimensions, &memory_allocator, &render_pass) {
                        swapchain = new_swapchain;
                        pipeline = get_terrain_pipeline(new_dimensions, &device, &render_pass);
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
                        render_pass: Some(subpass.clone().into()),
                        ..Default::default()
                    },
                ).unwrap();
                    
                let branch_uniforms = create_heightmap_uniform_buffer(&swapchain, &camera, GRASS_COLOUR, ROCK_COLOUR, GRASS_SLOPE_THRESHOLD, GRASS_BLEND_AMOUNT, &uniform_allocator);
                add_heightmap_terrain_draw_commands(&mut secondary_builder, &pipeline, &descriptor_set_allocator, &branch_uniforms, &lighting_uniforms, &mut world);
                
                
                builder.execute_commands(secondary_builder.build().unwrap()).unwrap().end_render_pass().unwrap();
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


fn single_pass_renderpass(
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
            ]
    ).unwrap()
}