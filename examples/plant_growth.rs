use std::{sync::Arc, time::{Duration, Instant}};
use synthetic_silviculture::{
    branch::*,
    plant::*,
    branch_development::*,
    branch_node::*,
    maths::vector_three::Vector3,
    graphics::{
        branch_mesh_gen::{update_next_mesh, check_for_force_update, MeshUpdateQueue},
        camera_maths::Camera,
        branch_graphics::*, 
        gui::*, 
        general_graphics::*
    },
    fixed_schedule::FixedSchedule,
    branch_prototypes::{BranchPrototypesSampler, BranchPrototypes, BranchPrototypeData, BranchPrototypeRef},
    environment::{create_gravity_resource, create_physical_age_time_step},
    general_update::*,
    plant_development::*,
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

    ///////////////////// graphics stuff /////////////////////
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup("branch render example".to_string());
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = gui_and_branch_renderpass(&device, &swapchain);
    let branch_subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    let gui_subpass = Subpass::from(render_pass.clone(), 1).unwrap();


    let mut camera = Camera{position: Vector3::new(-4.0, 3.0, 0.0), ..Default::default()};

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
    

    // uniforms
    let branch_uniform_buffer = create_branch_uniform_buffer(&memory_allocator);


    ///////////////////// ecs stuff /////////////////////
    let mut world = World::new();

    // gui
    add_world_gui_resources(&mut world, gui);

    

    // startup
    let mut startup_schedule = Schedule::default();
    startup_schedule.add_system(create_branch_resources_gui);

    // sceduling
    let mut gui_schedule = Schedule::default();
    gui_schedule.add_systems((draw_gui_objects, update_branch_resources.after(draw_gui_objects)));

    let mut graphics_update_schedule = Schedule::default();
    graphics_update_schedule.add_systems((check_for_force_update, update_next_mesh));

    let plant_update_schedule = get_plant_schedule(); // i put this in a seperate fn bc it was kinda long
    let mut fixed_plant_update = FixedSchedule::new(Duration::from_secs_f32(0.1), plant_update_schedule);


    // resources
    create_gravity_resource(&mut world, -Vector3::Y(), 0.05);
    add_world_branch_graphics_resources(&mut world);
    create_physical_age_time_step(&mut world, 0.75);

    // branch prototypes
    world.insert_resource(BranchPrototypes{
        prototypes: vec![
            BranchPrototypeData::new(
                25.0,
                vec![vec![2], vec![1, 2], vec![2, 1, 2]],
                vec![
                    Vector3::new(0.743, 0.371, 0.557),
                    Vector3::new(0.192, 0.962, 0.192),

                    Vector3::new(0.557, 0.743, 0.371),
                    Vector3::new(0.236, 0.943, 0.236),
                    Vector3::new(0.588, 0.784, 0.196),

                    Vector3::new(0.802, 0.535, 0.267),
                    Vector3::new(-0.535, 0.267, 0.802),
                    Vector3::new(-0.302, 0.905, 0.302),
                    Vector3::new(-0.333, 0.667, -0.667),
                    Vector3::new(0.301, 0.904, 0.301),
                ],
            ),
        ]
    });

    world.insert_resource(BranchPrototypesSampler::create(vec![([0, 255, 0], 10.0, 10.0)], (200, 200), 20.0, 20.0));
    world.insert_resource(PlantDeathRate::new(0.5));


    // plant
    let root_node_id = world.spawn(BranchNodeBundle{
        data: BranchNodeData{
            thickening_factor: 0.05,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let root_branch_id = world.spawn(BranchBundle{
        data: BranchData {
            root_node: Some(root_node_id),
            ..Default::default()
        },
        prototype: BranchPrototypeRef(0),
        ..Default::default()
    }).id();

    let plant_id = world.spawn(PlantBundle{
        growth_factors: PlantGrowthControlFactors{
            max_age: 200.0,
            max_vigor: 42.0,
            min_vigor: 2.0,
            apical_control: 0.62,
            growth_rate: 0.19,
            tropism_time_control: 0.38,
            max_branch_segment_length: 1.0,
            branch_segment_length_scaling_coef: 1.0,
            ..Default::default()
        },
        data: PlantData {
            root_node: Some(root_branch_id),
            ..Default::default()
        },
        ..Default::default()
    }).id();

    // mesh queue
    world.spawn(MeshUpdateQueue::new_from_single(plant_id));


    // run startup
    startup_schedule.run(&mut world);

    ///////////////////// run /////////////////////
    let mut last_frame_time = Instant::now();

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
                // fixed schedules
                let delta_time = last_frame_time.elapsed();
                fixed_plant_update.run(&mut world, delta_time);
                last_frame_time = Instant::now();
                

                // schedules
                gui_schedule.run(&mut world);
                graphics_update_schedule.run(&mut world);
                



                ///////////////////// graphics stuff /////////////////////

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


fn get_plant_schedule() -> Schedule {
    let mut plant_schedule = Schedule::default();

    plant_schedule.add_systems((
        update_branch_bounds,
        update_plant_bounds,
        update_plant_intersections,
        update_branch_intersections,
        calculate_branch_intersection_volumes,
        calculate_branch_light_exposure,
        calculate_growth_vigor,
        assign_growth_rates,
        step_physiological_age,
        update_branch_nodes,
        determine_create_new_branches,
        apply_system_buffers, // this makes sure nodes and branches have spawned
        assign_thicknesses,
        calculate_segment_lengths_and_tropism,
    ).chain());

    plant_schedule
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

