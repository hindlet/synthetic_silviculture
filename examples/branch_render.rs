//! An example of how to write the code to render a branch
//! 
//! - This example includes gui to control how detailed the branch is as well as how it is shaded
//! - There is also gui to control the branch's normal, note that if all are set to zero the branch will dissapear, this is fine as it will never actually happen




use std::sync::Arc;
use synthetic_silviculture::{
    branch::{BranchBundle, BranchData, BranchTag},
    plant::{PlantBundle, PlantData},
    branch_node::{BranchNodeBundle, BranchNodeData, BranchNodeConnectionData},
    maths::vector_three::Vector3,
    graphics::{
        branch_mesh_gen::{MeshUpdateQueue, update_next_mesh},
        camera_maths::Camera,
        branch_graphics::*, 
        gui::*, 
        general_graphics::*
    },
};
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents, CommandBufferInheritanceInfo, allocator::StandardCommandBufferAllocator},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Device,
    format::Format,
    pipeline::graphics::vertex_input::BuffersDefinition,
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

    // add stuff to the world
    let node_5_id = world.spawn(BranchNodeBundle {
        data: BranchNodeData {
            position: Vector3::new(1.5, 4.5, -1.0),
            thickness: 0.2,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let node_4_id = world.spawn(BranchNodeBundle {
        data: BranchNodeData {
            position: Vector3::new(2.0, 4.0, 1.0),
            thickness: 0.2,
            ..Default::default()
        },
        ..Default::default()
    }).id();
    
    let node_3_id = world.spawn(BranchNodeBundle {
        data: BranchNodeData {
            position: Vector3::new(-1.0, 2.2, 0.5),
            thickness: 0.25,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let node_2_id = world.spawn(BranchNodeBundle{
        data: BranchNodeData{
            position: Vector3::new(0.0, 2.5, 0.0),
            thickness: 0.3,
            ..Default::default()
        },
        connections: BranchNodeConnectionData{parent: None, children: vec![node_4_id, node_5_id]},
        ..Default::default()
    }).id();

    let node_1_id = world.spawn(BranchNodeBundle{
        data: BranchNodeData{
            position: Vector3::ZERO(),
            thickness: 0.4,
            ..Default::default()
        },
        connections: BranchNodeConnectionData{parent: None, children: vec![node_2_id, node_3_id]},
        ..Default::default()
    }).id();

    let branch_id = world.spawn(BranchBundle{
        data: BranchData{root_node: Some(node_1_id), normal: Vector3::Y(), ..Default::default()},
        ..Default::default()
    }).id();




    let plant_id = world.spawn(PlantBundle{data: PlantData{root_node: Some(branch_id), ..Default::default()}, ..Default::default()}).id();


    world.spawn(MeshUpdateQueue::new_from_single(plant_id));


    // do all the shader stuff
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup("branch_render".to_string());
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = gui_and_branch_renderpass(&device, &swapchain);
    let branch_subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    let gui_subpass = Subpass::from(render_pass.clone(), 1).unwrap();


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
    world.spawn(GUIData {
        name: "Branch Settings".to_string(),
        f32_sliders: vec![("normal-x".to_string(), 0.0, -1.0..=1.0), ("normal-y".to_string(), 1.0, -1.0..=1.0), ("normal-z".to_string(), 0.0, -1.0..=1.0)],
        ..Default::default()
    });

    // scheduling

    let mut startup_schedule = Schedule::default();
    startup_schedule.add_system(update_next_mesh);
    startup_schedule.add_system(create_branch_resources_gui);

    let mut gui_schedule = Schedule::default();
    gui_schedule.add_systems((draw_gui_objects, update_branch_resources.after(draw_gui_objects)));

    let mut update_schedule = Schedule::default();
    update_schedule.add_systems((update_next_mesh, update_branch_normal));

    
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


fn update_branch_normal(
    mut branch_query: Query<&mut BranchData, With<BranchTag>>,
    gui_query: Query<&GUIData>,
) {
    let mut branch = branch_query.single_mut();
    for gui in gui_query.iter() {
        if gui.name == "Branch Settings" {
            let mut normal = Vector3::new(gui.f32_sliders[0].1, gui.f32_sliders[1].1, gui.f32_sliders[2].1);
            normal.normalise();
            branch.normal = normal;
        }
    }
}

fn gui_and_branch_renderpass(
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