use synthetic_silviculture::branch::{BranchBundle, BranchData};
use synthetic_silviculture::graphics::branch_mesh_gen::{MeshUpdateQueue, update_next_mesh};
use synthetic_silviculture::general::vector_three::Vector3;
use synthetic_silviculture::graphics::camera_maths::Camera;
use synthetic_silviculture::graphics::{branch_graphics::*, gui::*, general_graphics::*};
use synthetic_silviculture::plant::{PlantBundle, PlantData};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::swapchain::{SwapchainCreateInfo, SwapchainCreationError, acquire_next_image, SwapchainPresentInfo, AcquireError};
use vulkano::sync::{self, GpuFuture};
use vulkano::sync::FlushError;
use winit::event::ElementState;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};
use bevy_ecs::prelude::*;
use synthetic_silviculture::branch_node::*;


fn main() {
    let mut world = World::new();

    // add stuff to the world
    add_world_branch_graphics_resources(&mut world);

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
        data: BranchData{root_node: Some(node_1_id), ..Default::default()},
        ..Default::default()
    }).id();

    let plant = world.spawn(PlantBundle{data: PlantData{root_node: Some(branch_id), ..Default::default()}, ..Default::default()}).id();

    world.spawn(MeshUpdateQueue {
        0: vec![plant]
    });


    // do all the shader stuff
    let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup();
    let (mut swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
    let render_pass = get_renderpass(&device, &swapchain);

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

    let mut startup_schedule = Schedule::default();
    #[derive(StageLabel)]
    struct StartupStage;
    startup_schedule.add_stage(StartupStage, SystemStage::parallel());
    startup_schedule.add_system_to_stage(StartupStage, update_next_mesh);
    
    startup_schedule.run_once(&mut world);

    // uniforms
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
                        .vertex::<Vertex>()
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
                    

                add_branch_draw_commands(&mut builder, &pipeline, &descriptor_set_allocator, &branch_uniforms, &memory_allocator, &mut world);

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