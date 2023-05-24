use super::{
    fixed_schedule::FixedSchedule,
    branches::{
        branch::*,
        branch_development::*,
        branch_prototypes::*,
        branch_node::*,
    },
    plants::{
        plant::*,
        plant_development::*,
    },
    general_update::*,
    environment::*,
    terrain::*,
    light_cells::*,
    maths::{
        vector_three::Vector3,
    },
    graphics::{
        general_graphics::*,
        branch_graphics::*,
        branch_mesh_gen::*,
        gui::*,
        camera_maths::Camera,
        terrain_graphics::*,
    },
};
use egui_winit_vulkano::Gui;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::Window, platform::run_return::EventLoopExtRunReturn,
};
use std::{time::{Duration, Instant}, sync::Arc, cell::RefCell, rc::Rc, borrow::BorrowMut};
use bevy_ecs::{prelude::*, system::SystemState};
use vulkano::{
    render_pass::{RenderPass, Subpass, Framebuffer},
    device::{Device, Queue},
    swapchain::{Swapchain, Surface, acquire_next_image, AcquireError, SwapchainCreateInfo, SwapchainPresentInfo},
    format::Format,
    memory::allocator::{StandardMemoryAllocator, GenericMemoryAllocator, FreeListAllocator, AllocationCreateInfo, MemoryUsage},
    image::SwapchainImage, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CommandBufferInheritanceInfo, RenderPassBeginInfo, SubpassContents, allocator::StandardCommandBufferAllocator},
    pipeline::{GraphicsPipeline, graphics::{vertex_input::Vertex, input_assembly::InputAssemblyState, viewport::{ViewportState, Viewport}}, PipelineLayout},
    buffer::{Subbuffer, BufferContents},
    sync::{self, GpuFuture, FlushError},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    buffer::{Buffer, BufferUsage, BufferCreateInfo},
};


enum TerrainType {
    Absent,
    Flat,
    Bumpy
}

enum OutputType {
    Absent,
    Data,
    Meshes,
    All,
}

#[derive(Clone)]
pub struct TreeAppOutput {
    pub data: Option<Vec<([f32; 3], Vec<([f32; 3], f32)>, Vec<(usize, usize)>)>>,
    pub meshes: Option<()>,
}

impl Default for TreeAppOutput {
    fn default() -> Self {
        TreeAppOutput {data: None, meshes: None}
    }
}


#[derive(Debug)]
pub struct GraphicsAppBuilder {
    // features
    has_terrain: bool,
    has_gui: bool,
    output: u32,


    // settings
    window_title: String,
    terrain_settings: Option<(f32, Vector3, Option<(u32, f32, String)>)>, // size, centre, verts per side, height mult, path
    terrain_graphics_settings: Option<([f32; 3], [f32; 3], f32, f32)>,
    lights: Option<(Vec<([f32; 3], f32)>, Vec<([f32; 3], f32)>)>,
    gravity_strength: Option<f32>,
    prototype_conditions: Option<(Vec<(f32, f32)>, f32, f32)>,
    prototypes: Option<Vec<(f32, Vec<Vec<u32>>,  Vec<[f32; 3]>)>>,
    time_step: Option<f32>,
    cell_settings: Option<(u32, f32)>,
    plant_death_rate: Option<f32>,
    start_plants: u32,
    has_plants: bool,

}


pub struct LoopedTreeApp {
    world: World,
    update_schedule: Schedule,
    output: OutputType
}


pub struct GraphicsTreeApp {
    world: World,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    event_loop: EventLoop<()>,
    memory_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,


    frame_schedule: Schedule,
    update_schedule: FixedSchedule,
    output: OutputType,

    graphics_pass: Subpass,
    branch_pipeline: Arc<GraphicsPipeline>,
    terrain: TerrainType,
    terrain_pipeline: Option<Arc<GraphicsPipeline>>,
    terrain_settings: Option<([f32; 3], [f32; 3], f32, f32)>,
    lights: (Vec<([f32; 3], f32)>, Vec<([f32; 3], f32)>),
    gui: Option<Gui>,

    camera_state: ([f32; 3], [f32; 3]),

}



impl GraphicsTreeApp {
    /// creates a new app builder to add functionality to with a given window title
    /// 
    /// WARNING - Only works if the vulkan graphics library is installed on your system
    pub fn new(window_title: String) -> GraphicsAppBuilder{
        GraphicsAppBuilder {
            has_terrain: false,
            has_gui: false,
            output: 0,

            window_title,
            terrain_settings: None,
            terrain_graphics_settings: None,
            gravity_strength: None,
            lights: None,
            prototypes: None,
            prototype_conditions: None,
            time_step: None,
            cell_settings: None,
            plant_death_rate: None,
            start_plants: 1,
            has_plants: false,
        }
    }


    pub fn run(mut self) -> TreeAppOutput{

        let mut update_schedule = self.update_schedule;
        let mut frame_schedule = self.frame_schedule;
        let mut world = self.world;
        let device = self.device.clone();
        let queue = self.queue.clone();
        let surface = self.surface.clone();
        let mut swapchain = self.swapchain.clone();
        let memory_allocator = self.memory_allocator.clone();
        let render_pass = self.render_pass;
        let mut framebuffers = self.framebuffers.clone();
        let graphics_pass = self.graphics_pass.clone();
        let mut branch_pipeline = self.branch_pipeline.clone();
        let mut terrain_pipeline = self.terrain_pipeline.clone();
        

        let mut camera = Camera::new(Some(self.camera_state.0), Some(self.camera_state.1), None, None);

        let uniform_allocator = create_uniform_buffer_allocator(&self.memory_allocator);
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(self.device.clone());
        let command_buffer_allocator = StandardCommandBufferAllocator::new(self.device.clone(), Default::default());

        let branch_lighting_uniforms = get_branch_light_buffers(self.lights.0.clone(), self.lights.1.clone(), &self.memory_allocator);

        let terrain_lighting_uniforms = {
            match self.terrain {
                TerrainType::Absent => {None},

                TerrainType::Flat => {
                    None
                }

                TerrainType::Bumpy => {
                    Some(get_heightmap_light_buffers(self.lights.0.clone(), self.lights.1.clone(), &self.memory_allocator))
                }
            }
        };


        let output = Rc::new(RefCell::new(TreeAppOutput{data: None, meshes: None}));
        let output_ref = output.clone();


        let mut recreate_swapchain = false;
        let mut last_frame_time = Instant::now();
        let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
        
        self.event_loop.run_return(move |event, _, control_flow| {
            match event {
                
                Event::WindowEvent { window_id: _, event } => {
                    // pass things to gui
                    if let Some(gui) = self.gui.as_mut() {
                        pass_winit_event_to_gui(gui, &event);
                    }
                    // check for resize or close
                    match event {
                        WindowEvent::Resized(_) => {
                            recreate_swapchain = true
                        }
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            match self.output {
                                OutputType::Absent => {}
                                _ => {output_ref.replace(TreeAppOutput{data: Some(data_output(&mut world)), meshes: None});}
                            }
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
                },


                Event::MainEventsCleared => {
                    // fixed schedules
                    let delta_time = last_frame_time.elapsed();
                    update_schedule.run(&mut world, delta_time);
                    camera.do_move(delta_time);
                    last_frame_time = Instant::now();
                    

                    // schedules
                    if let Some(gui) = self.gui.as_mut() {
                        run_gui_commands(&mut world, gui);
                    }
                    frame_schedule.run(&mut world);



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
                            branch_pipeline = get_branch_pipeline(new_dimensions, &device, &render_pass, 0);
                            match self.terrain {
                                TerrainType::Absent => {},
                                TerrainType::Flat => {},
                                TerrainType::Bumpy => {terrain_pipeline = Some(get_terrain_pipeline(new_dimensions, &device, &render_pass, 0))}
                            }
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

                    //// builder setup
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
                        CommandBufferUsage::OneTimeSubmit,
                        CommandBufferInheritanceInfo {
                            render_pass: Some(graphics_pass.clone().into()),
                            ..Default::default()
                        },
                    ).unwrap();


                    ///// terrain graphics
                    match self.terrain {
                        TerrainType::Absent => {},

                        TerrainType::Flat => {},

                        TerrainType::Bumpy => {
                            let terrain_uniforms = create_heightmap_uniform_buffer(&swapchain, &camera, self.terrain_settings.unwrap().0, self.terrain_settings.unwrap().1, self.terrain_settings.unwrap().2, self.terrain_settings.unwrap().3, &uniform_allocator);
                            add_heightmap_terrain_draw_commands(&mut secondary_builder, terrain_pipeline.as_ref().unwrap(), &descriptor_set_allocator, &terrain_uniforms, terrain_lighting_uniforms.as_ref().unwrap(), 0, &mut world);
                        }
                    }

                    ////// branch_graphics
                    let branch_uniforms = create_branch_uniform_buffer(&swapchain, &camera, &uniform_allocator);
                    add_branch_draw_commands(&mut secondary_builder, &branch_pipeline, &descriptor_set_allocator, &branch_uniforms, &branch_lighting_uniforms, &mut world);
                    

                    builder.execute_commands(secondary_builder.build().unwrap()).unwrap();

                    ////// gui graphics
                    if let Some(gui) = self.gui.as_mut() {
                        builder.next_subpass(SubpassContents::SecondaryCommandBuffers).unwrap();
                        let gui_command_buffer = get_gui_resource_commands(gui, dimensions.into());
                        builder.execute_commands(gui_command_buffer).unwrap();
                    }

                    builder.end_render_pass().unwrap();
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
        });

        let app_out = RefCell::take(&output);
        app_out
    }
    
}



impl GraphicsAppBuilder {
    /// features
    

    /// allows plants to exist in the world
    pub fn with_plants(&mut self) -> &mut GraphicsAppBuilder {
        self.has_plants = true;
        self
    }

    /// allows the app to draw flat terrain, overrides previous terrain settings
    pub fn with_flat_terrain(&mut self, size: f32, centre: [f32; 3], colour: [f32; 3]) -> &mut GraphicsAppBuilder {
        self.has_terrain = true;
        self.terrain_settings = Some((size, centre.into(), None));
        self.terrain_graphics_settings = Some((colour, [0.0, 0.0, 0.0], 0.0, 0.0));
        self
    }

    /// allows the app to draw heightmap based terrain, overrides previous terrain settings
    pub fn with_heightmap_terrain(&mut self, size: f32, centre: [f32; 3], verts_per_side: u32, height_scale: f32, heightmap_path: &str, grass_colour: [f32; 3], rock_colour: [f32; 3], grass_slope_threshold: f32, grass_blend_amount: f32) -> &mut GraphicsAppBuilder{
        self.has_terrain = true;
        self.terrain_settings = Some((size, centre.into(), Some((verts_per_side, height_scale, heightmap_path.into()))));
        self.terrain_graphics_settings = Some((grass_colour, rock_colour, grass_slope_threshold, grass_blend_amount));
        self
    }

    /// allows the app to draw branch graphics settings gui, requires rendering to be enabled to have an effect
    pub fn with_branch_graphics_gui(&mut self) -> &mut GraphicsAppBuilder {
        self.has_gui = true;
        self
    }

    /// sets the light data used for the graphics drawing, currently only point lights and directional lights are supported
    /// 
    /// - Point Light: (position, intensity)
    /// - Directional Light: (direction, intensity)
    pub fn set_lights(&mut self, point_lights: Vec<([f32; 3], f32)>, directional_lights: Vec<([f32; 3], f32)>) -> &mut GraphicsAppBuilder {
        self.lights = Some((point_lights, directional_lights));
        self
    }

    /// sets the output type from the app, 0 1 2 or 3, any other number will default to 0
    /// 
    /// - 0 - no output
    /// - 1 - outputs data about branch nodes: such as node position and thickness; and connection data: which nodes are connected to each other
    /// - 2 - meshes: creates meshes for the plants and outputs the data for them: vertices, normals, positions and indices
    /// - 3 - data and meshes, outputs the data for option 1 and 2
    pub fn set_output_type(&mut self, mut output: u32) -> &mut GraphicsAppBuilder {
        if output > 3 {output = 0;}
        self.output = output;
        self
    }

    /// simulation settings
    
    /// sets the strength of gravity and the stength, a negative strength represents phototropism, a positive gravitropism
    /// 
    /// the default value is 1
    pub fn set_gravity(&mut self, strength: f32) -> &mut GraphicsAppBuilder {
        self.gravity_strength = Some(strength);
        self
    }

    /// sets the branch presets used for the simulation, overrides the default set of branches used
    pub fn set_branch_presets(&mut self, prototypes: Vec<(f32, Vec<Vec<u32>>, Vec<[f32; 3]>)>, conditions: (Vec<(f32, f32)>, f32, f32)) -> &mut GraphicsAppBuilder {
        self.prototypes = Some(prototypes);
        self.prototype_conditions = Some(conditions);
        self
    }

    /// sets the plant species used for the simulation, overrides default set of plant species used
    /// 
    /// Plant species are used for initial plant spawning and spawning of new plants without seeding
    pub fn set_plant_species(&mut self) -> &mut GraphicsAppBuilder {


        self
    }


    /// set how many plants are used in the simulation, currently changes nothing
    pub fn set_initial_plant_num(&mut self, num: u32) -> &mut GraphicsAppBuilder {
        self.start_plants = num;

        self
    }


    /// sets the size of the cell grid used to shadow distribution, the default is 1m^3. And the number of cells above or below that will be checked for shadow distribution purposes
    pub fn set_shadow_cell_data(&mut self, size: f32, check_height: u32) -> &mut GraphicsAppBuilder {
        self.cell_settings = Some((check_height, size.abs()));

        self
    }


    /// sets the physical time step used for plant aging, defaults to 1.0
    pub fn set_time_step(&mut self, step: f32) -> &mut GraphicsAppBuilder {
        self.time_step = Some(step.abs());

        self
    }

    /// sets the rate at which plants die, defaults to 1.0
    pub fn set_plant_death_rate(&mut self, rate: f32) -> &mut GraphicsAppBuilder {
        self.plant_death_rate = Some(rate.abs());

        self
    }




    /// builds and returns the app for running
    pub fn build(&mut self) -> GraphicsTreeApp{

        // transform data
        let output = {
            if self.output == 0 {OutputType::Absent}
            else if self.output == 1 {OutputType::Data}
            else if self.output == 2{OutputType::Meshes}
            else {OutputType::All}
        };
        
        let gravity_strength = self.gravity_strength.unwrap_or(DEFAULT_GRAVITY_STRENGTH);
        let time_step = self.time_step.unwrap_or(DEFAULT_TIMESTEP);
        let branch_conditions = self.prototype_conditions.clone().unwrap_or(DEFAULT_BRANCH_CONTIDITIONS);
        let branch_types = self.prototypes.clone().unwrap_or(DEFAULT_BRANCH_TYPES);
        let cell_settings = self.cell_settings.unwrap_or(DEFAULT_CELL_SETTINGS);
        let plant_death_rate = self.plant_death_rate.unwrap_or(DEFAULT_PLANT_DEATH_RATE);



        ///////////////// world
        let mut world = World::new();

        ///////////////// resources
        create_gravity_resource(&mut world, [0, -1, 0], gravity_strength);
        create_physical_age_time_step(&mut world, time_step);
        world.insert_resource(BranchPrototypesSampler::create(branch_conditions.0, SAMPLER_SIZE, branch_conditions.1, branch_conditions.2));
        world.insert_resource(BranchPrototypes::new(branch_types));
        world.insert_resource(LightCells::new(cell_settings.0 as i32, cell_settings.1));
        world.insert_resource(PlantDeathRate::new(plant_death_rate));

        let mut update_schedule = Schedule::new();

        // spawn initial plant
        // TODO: change how it is decided
        
        // plant
        if self.has_plants {
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

            world.spawn(PlantBundle{
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
            });

            // mesh queue
            world.spawn(MeshUpdateQueue::new_from_single(root_branch_id));
            update_schedule.add_systems((
                // update_branch_bounds,
                // update_plant_bounds,
                // update_plant_intersections,
                calculate_branch_light_exposure,
                calculate_growth_vigor,
                assign_growth_rates,
                step_physiological_age,
                update_branch_nodes,
                determine_create_new_branches,
                apply_system_buffers, // this makes sure nodes and branches have spawned
                assign_thicknesses,
                calculate_segment_lengths_and_tropism,
                update_branch_data_buffers,
            ).chain());
    
        }
        else {world.spawn(MeshUpdateQueue::new());}




        

        ///////////////// sceduling


        
        

        ////////// rendered stuff


        /////// base graphics
        let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup(self.window_title.clone());
        let (swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
        let render_pass = if self.has_gui {double_pass_renderpass(&device, &swapchain)} else {single_pass_renderpass(&device, &swapchain)};
        let (framebuffers, window_dimensions) = get_framebuffers(&memory_allocator, &swapchain_images, &render_pass);


        add_world_branch_graphics_resources(&mut world, memory_allocator.clone());
        

        /////// subpasses and pipelines
        

        let (terrain_type, terrain_pipeline) = {
            if self.has_terrain {     
                let pipeline = Some(get_terrain_pipeline(window_dimensions, &device, &render_pass, 0));
                let settings = self.terrain_settings.clone().unwrap();
                if settings.2.is_none() {
                    spawn_flat_terrain(settings.0, settings.1, &mut world);
                    create_terrain_mesh_buffers(&memory_allocator, &mut world);
                    (TerrainType::Flat, pipeline)
                }
                else {
                    let subsettings = settings.2.clone().unwrap();
                    spawn_heightmap_terrain(settings.0, subsettings.0, subsettings.1, settings.1, subsettings.2, &mut world);
                    create_terrain_mesh_buffers(&memory_allocator, &mut world);
                    (TerrainType::Bumpy, pipeline)
                }
            }
            else {(TerrainType::Absent, None)}
        };

        let branch_pipeline = get_branch_pipeline(window_dimensions, &device, &render_pass, 0);

        let graphics_pass = Subpass::from(render_pass.clone(), 0).unwrap();
        let gui = {
            if self.has_gui {
                let subpass = Subpass::from(render_pass.clone(), 1).unwrap();
                Some(create_gui_from_subpass(&event_loop, &surface, &queue, &subpass))
            }
            else {None}
        };




        let mut frame_schedule = Schedule::default();
        frame_schedule.add_systems((check_for_force_update, update_next_mesh));


        // startrup
        let mut startup_schedule = Schedule::new();

        startup_schedule.add_system(init_branch_mesh_buffers_res);

        startup_schedule.run(&mut world);


        GraphicsTreeApp{
            world: world,
            device,
            queue,
            surface,
            event_loop,
            memory_allocator,
            swapchain,
            render_pass,
            framebuffers,

            frame_schedule,
            update_schedule: FixedSchedule::new(Duration::from_secs_f32(0.1), update_schedule),
            output,

            
            graphics_pass,
            branch_pipeline,
            terrain: terrain_type,
            terrain_pipeline,
            terrain_settings: self.terrain_graphics_settings,
            gui,
            // this made me want to give up on the whole project - 2023-05-21: the second value was [0.0; 3] which caused no 3d graphics to render, I had been debugging for 3 days
            camera_state: ([-10.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            lights: self.lights.clone().unwrap_or(DEFAULT_LIGHTS)
        }
    }

    
}


//////////////////// consts
const SAMPLER_SIZE: (u32, u32) = (500, 500);
const DEFAULT_GRAVITY_STRENGTH: f32 = 1.0;
const DEFAULT_TIMESTEP: f32 = 1.0;
const DEFAULT_BRANCH_TYPES: Vec<(f32, Vec<Vec<u32>>, Vec<[f32; 3]>)> = Vec::new();
const DEFAULT_BRANCH_CONTIDITIONS: (Vec<(f32, f32)>, f32, f32) = (Vec::new(), 1.0, 1.0);
const DEFAULT_CELL_SETTINGS: (u32, f32) = (5, 0.5);
const DEFAULT_PLANT_DEATH_RATE: f32 = 1.0;
const DEFAULT_LIGHTS: (Vec<([f32; 3], f32)>, Vec<([f32; 3], f32)>) = (Vec::new(), Vec::new());




//////// render pass generators
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
                { color: [color], depth_stencil: {depth}, input: [] }
            ]
    ).unwrap()
}


fn double_pass_renderpass(
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
                { color: [color], depth_stencil: {depth}, input: [] },
                { color: [color], depth_stencil: {depth}, input: [] }
            ]
    ).unwrap()
}


fn data_output(
    world: &mut World,
) -> Vec<([f32; 3], Vec<([f32; 3], f32)>, Vec<(usize, usize)>)>{

    let mut state: SystemState<(
        Query<&BranchNodeData, With<BranchNodeTag>>,
        Query<&BranchNodeConnectionData, With<BranchNodeTag>>,
        Query<&BranchData, With<BranchTag>>,
        Query<&BranchConnectionData, With<BranchTag>>,
        Query<&PlantData, With<PlantBounds>>,
    )> = SystemState::new(world);

    let (node_data, node_connections, branch_data, branch_connections, plant_data) = state.get(world);

    // plant positions : node position and thickness : node connections
    let mut data: Vec<([f32; 3], Vec<([f32; 3], f32)>, Vec<(usize, usize)>)> = Vec::new();
    for plant in plant_data.iter() {
        if plant.root_node.is_none() {continue;}
        
        let position: [f32; 3] = plant.position.into();

        let mut plant_data: (Vec<([f32; 3], f32)>, Vec<(usize, usize)>) = (Vec::new(), Vec::new());

        let mut current_nodes: usize = 0;

        for id in get_branches_base_to_tip(&branch_connections, plant.root_node.unwrap()) {
            if let Ok(branch) = branch_data.get(id) {
                if branch.root_node.is_none() {continue;}

                let (positions, thicknesses, pairs) = get_node_data_and_connections_base_to_tip(&node_connections, &node_data, branch.root_node.unwrap());
                let mut sub_data: Vec<([f32; 3], f32)> = Vec::new();
                for i in 0..positions.len() {
                    sub_data.push((positions[i].into(), thicknesses[i]));
                }
                let start_point = plant_data.0.iter().position(|&x| x == sub_data[0]).unwrap_or(0);
                if start_point != 0 {current_nodes -= 1; sub_data.remove(0);}

                let mut new_pairs: Vec<(usize, usize)> = Vec::new();
                for pair in pairs {
                    let one = if pair.0 == 0 {start_point} else {pair.0 + current_nodes};
                    let two = if pair.0 == 0 {start_point} else {pair.0 + current_nodes};
                    new_pairs.push((one, two));
                }
                plant_data.0.append(&mut sub_data);
                plant_data.1.append(&mut new_pairs);
                current_nodes = plant_data.0.len() - 1;
            }
        }
        data.push((position, plant_data.0, plant_data.1));
    }


    data
}



