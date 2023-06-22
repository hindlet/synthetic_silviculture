use super::super::{
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
        plant_selection::*,
    },
    environment::*,
    environment::{
        terrain::*,
        light_cells::*,
        params::*,
    },
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
    debug::*,
};
use super::*;
use egui_winit_vulkano::Gui;
use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode},
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





pub struct GraphicsAppBuilder {
    // features
    has_terrain: bool,
    has_gui: bool,
    output: u32,

    // settings
    window_title: String,
    terrain_settings: Option<(f32, Vector3, Option<(u32, f32, String)>)>, // size, centre, verts per side, height mult, path
    terrain_graphics_settings: Option<([f32; 3], [f32; 3], f32, f32)>,
    light: Option<([f32; 3], f32)>,
    branch_render_settings: Option<(u32, bool)>,

    gravity_strength: Option<f32>,
    time_step: Option<f32>,
    cell_settings: Option<(u32, f32)>,
    plant_death_rate: Option<f32>,
    environmental_params: Option<(f32, f32, f32)>, // temp at y=0, temp falloff, moisture

    prototype_conditions: Option<(Vec<(f32, f32)>, f32, f32)>,
    prototypes: Option<Vec<(f32, Vec<Vec<u32>>,  Vec<[f32; 3]>)>>,
    start_plants: u32,
    plant_species: Option<Vec<((GrowthControlSettingParams, PlasticitySettingParams), (f32, f32, f32, f32))>>,
    has_seeding: bool,


}




/// p to pause
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
    paused: bool,

    graphics_pass: Subpass,
    branch_pipeline: Arc<GraphicsPipeline>,
    terrain: TerrainType,
    terrain_pipeline: Option<Arc<GraphicsPipeline>>,
    terrain_settings: Option<([f32; 3], [f32; 3], f32, f32)>,
    light: ([f32; 3], f32),
    gui: Option<Gui>,

    camera_state: ([f32; 3], [f32; 3]),

}



impl GraphicsTreeApp {
    /// creates a new app builder to add functionality to with a given window title
    /// 
    /// WARNING - Only works if the vulkan graphics library is installed on your system
    /// 
    /// Please note that if no terrain is specified, plants will not be able to reproduce and initial plants will be spawned on a 50m by 50m square at y=0
    pub fn new(window_title: String) -> GraphicsAppBuilder{
        GraphicsAppBuilder {
            has_terrain: false,
            has_gui: false,
            output: 0,

            window_title,
            terrain_settings: None,
            terrain_graphics_settings: None,
            gravity_strength: None,
            light: None,
            branch_render_settings: None,
            prototypes: None,
            prototype_conditions: None,
            time_step: None,
            cell_settings: None,
            plant_death_rate: None,
            start_plants: 0,

            environmental_params: None,
            plant_species: None,
            has_seeding: false
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


        let output = Rc::new(RefCell::new(TreeAppOutput{data: None, meshes: None}));
        let output_ref = output.clone();


        let mut recreate_swapchain = false;
        let mut last_frame_time = Instant::now();
        let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

        let mut prev_pause_key_state = ElementState::Released;
        
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
                            if keycode == VirtualKeyCode::P && state == ElementState::Pressed && prev_pause_key_state == ElementState::Released{
                                self.paused ^= true;
                            }
                            if keycode == VirtualKeyCode::P {
                                prev_pause_key_state = state;
                            }

                            camera.process_key(keycode, state == ElementState::Pressed);
                        }
                        _ => (),
                    }
                },


                Event::MainEventsCleared => {
                    // fixed schedules
                    let delta_time = last_frame_time.elapsed();
                    if !self.paused {
                        update_schedule.run(&mut world, delta_time);
                    }
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
                                TerrainType::Flat => {terrain_pipeline = Some(get_flat_terrain_pipeline(new_dimensions, &device, &render_pass, 0))},
                                TerrainType::Bumpy => {terrain_pipeline = Some(get_heightmap_terrain_pipeline(new_dimensions, &device, &render_pass, 0))}
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

                        TerrainType::Flat => {
                            let terrain_uniforms = create_flat_uniform_buffer(&swapchain, &camera, self.light, self.terrain_settings.unwrap().0, &uniform_allocator);
                            add_flat_terrain_draw_commands(&mut secondary_builder, terrain_pipeline.as_ref().unwrap(), &descriptor_set_allocator, &terrain_uniforms, 0, &mut world);
                        },

                        TerrainType::Bumpy => {
                            let terrain_uniforms = create_heightmap_uniform_buffer(&swapchain, &camera, self.light, self.terrain_settings.unwrap().0, self.terrain_settings.unwrap().1, self.terrain_settings.unwrap().2, self.terrain_settings.unwrap().3, &uniform_allocator);
                            add_heightmap_terrain_draw_commands(&mut secondary_builder, terrain_pipeline.as_ref().unwrap(), &descriptor_set_allocator, &terrain_uniforms, 0, &mut world);
                        }
                    }

                    ////// branch_graphics
                    let branch_uniforms = create_branch_uniform_buffer(&swapchain, &camera, self.light, &uniform_allocator);
                    add_branch_draw_commands(&mut secondary_builder, &branch_pipeline, &descriptor_set_allocator, &branch_uniforms, &mut world);
                    

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

    /// sets the light data used for the graphics drawing, currently only directional lights are supported
    /// 
    /// - Directional Light: (direction, intensity)
    pub fn set_light(&mut self, directional_light: ([f32; 3], f32)) -> &mut GraphicsAppBuilder {
        self.light = Some(directional_light);
        self
    }

    /// sets the number of polygons used to construct branch meshes and if branches are flat shaded
    /// 
    /// Defaults to 3 faces and smooth shaded
    /// Enable branch graphics gui to change while running
    pub fn set_branch_mesh_settings(&mut self, faces: u32, flat_shaded: bool) -> &mut GraphicsAppBuilder {
        self.branch_render_settings = Some((faces.max(3), flat_shaded));

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
    pub fn set_plant_species(&mut self, species: Vec<((GrowthControlSettingParams, PlasticitySettingParams), (f32, f32, f32, f32))>) -> &mut GraphicsAppBuilder {
        self.plant_species = Some(species);

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


    /// sets the physical time step used for plant aging in years per step, defaults to 1.0
    /// 
    /// This is how many years will pass per second
    pub fn set_time_step(&mut self, step: f32) -> &mut GraphicsAppBuilder {
        self.time_step = Some(step.abs());

        self
    }

    /// sets the rate at which plants die, defaults to 1.0
    pub fn set_plant_death_rate(&mut self, rate: f32) -> &mut GraphicsAppBuilder {
        self.plant_death_rate = Some(rate.abs());

        self
    }


    /// sets the environmental parameters used by the simulation
    /// 
    /// - Temperature: (Degrees Celsius at a height of y=0, rate of temperature decrease away from y=0)
    /// - Moisture: Average annual precipitation, cm
    pub fn set_environmental_parameters(&mut self, temperature: (f32, f32), moisture: f32) -> &mut GraphicsAppBuilder {
        self.environmental_params = Some((temperature.0, temperature.1, moisture));

        self
    }

    /// - Enables plant seeding, meaning that plants will reproduce
    /// - This is disabled by default
    /// - Has no effect without initial plants
    pub fn enable_seeding(&mut self) -> &mut GraphicsAppBuilder {
        self.has_seeding = true;

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
        let time_step = self.time_step.unwrap_or(DEFAULT_TIMESTEP) / 10.0;
        let branch_conditions = self.prototype_conditions.clone().unwrap_or(DEFAULT_BRANCH_CONTIDITIONS);
        let branch_types = self.prototypes.clone().unwrap_or(DEFAULT_BRANCH_TYPES);
        let cell_settings = self.cell_settings.unwrap_or(DEFAULT_CELL_SETTINGS);
        let plant_death_rate = self.plant_death_rate.unwrap_or(DEFAULT_PLANT_DEATH_RATE);
        let has_plants = self.start_plants > 0;
        let plant_species = self.plant_species.clone().unwrap_or(DEFAULT_PLANT_SPECIES);
        let environmental_params = self.environmental_params.unwrap_or(DEFAULT_ENVIRONMENTAL_PARAMS);
        
        let branch_sampler = BranchPrototypesSampler::create(branch_conditions.0, SAMPLER_SIZE, branch_conditions.1, branch_conditions.2);
        let plant_species_sampler = PlantSpeciesSampler::new(plant_species, time_step);

        ///////////////// world
        let mut world = World::new();

        ///////////////// resources
        create_gravity_resource(&mut world, [0, -1, 0], gravity_strength);
        create_physical_age_time_step(&mut world, time_step);

        
        
        world.insert_resource(BranchPrototypes::new(branch_types));
        world.insert_resource(LightCells::new(cell_settings.0 as i32, cell_settings.1));
        world.insert_resource(PlantDeathRate::new(plant_death_rate));


        let (terrain_type, plant_spawning_bounds, terrain_collider_ref) = {
            if self.has_terrain {     
                let settings = self.terrain_settings.clone().unwrap();
                if settings.2.is_none() {
                    let plant_spawn_bounds = spawn_flat_terrain(settings.0, settings.1, &mut world);
                    (TerrainType::Flat, plant_spawn_bounds.0, plant_spawn_bounds.1)
                }
                else {
                    let subsettings = settings.2.clone().unwrap();
                    let plant_spawn_bounds = spawn_heightmap_terrain(settings.0, subsettings.0, subsettings.1, settings.1, subsettings.2, &mut world);
                    (TerrainType::Bumpy, plant_spawn_bounds.0, plant_spawn_bounds.1)
                }
            }
            else {
                let plant_spawn_bounds = spawn_flat_terrain(DEFAULT_TERRAIN.0, DEFAULT_TERRAIN.1, &mut world);
                (TerrainType::Absent, plant_spawn_bounds.0, plant_spawn_bounds.1)
            }
        };




        let mut update_schedule = Schedule::new();

        // spawn initial plant(s)

        let mut initial_plant_data = Vec::new();
        let mut rng = thread_rng();

        for _ in 0..self.start_plants {

            let (x, z) = (rng.gen_range(plant_spawning_bounds.1.clone()), rng.gen_range(plant_spawning_bounds.2.clone()));

            let hit = terrain_collider_ref.check_ray([x, plant_spawning_bounds.0 + 5.0, z], [0, -1, 0], None).unwrap();

            initial_plant_data.push((plant_species_sampler.get_plant(environmental_params.0 + hit.hit_position.y * environmental_params.1, environmental_params.2), hit.hit_position))
        }

        let mut root_ids = Vec::new();
        for data in initial_plant_data {

            if let (Some((spawn_data, climate_adapt)), pos) = data {

                let root_node_id = world.spawn(BranchNodeBundle{
                    data: BranchNodeData{
                        thickening_factor: spawn_data.0.thickening_factor,
                        ..Default::default()
                    },
                    ..Default::default()
                }).id();

                let root_branch_id = world.spawn(BranchBundle{
                    data: BranchData {
                        root_node: Some(root_node_id),
                        root_position: pos.into(),
                        ..Default::default()
                    },
                    prototype: BranchPrototypeRef(branch_sampler.get_prototype_index(spawn_data.0.apical_control, branch_conditions.2)),
                    ..Default::default()
                }).id();

                world.spawn(PlantBundle{
                    growth_factors: spawn_data.0,
                    data: PlantData {
                        root_node: Some(root_branch_id),
                        position: pos.into(),
                        climate_adaption: climate_adapt,
                        ..Default::default()
                    },
                    plasticity_params: spawn_data.1,
                    ..Default::default()
                });

                root_ids.push(root_branch_id);
            }
        }
        world.insert_resource(branch_sampler);
        
        // plant
        if has_plants {

            if root_ids.len() == 0 {panic!("No intial plants generated")}
            
            // mesh queue
            world.spawn(MeshUpdateQueue::new_from_many(root_ids, 5));

            update_schedule.add_systems((
                update_branch_bounds,
                update_plant_bounds,
                update_plant_intersections,
                update_branch_intersections,
                calculate_branch_intersection_volumes,
                // debug_log_branches,
                // debug_log_cells,
                step_plant_age,
                calculate_branch_light_exposure,
                calculate_growth_vigor,
                trim_branches,
                apply_system_buffers, // this makes sure nodes and branches have been removed
                remove_dead_connections,
                assign_growth_rates,
                step_physiological_age,
            ).chain());

            update_schedule.add_systems((
                update_branch_nodes,
                apply_system_buffers, // this makes sure new nodes are spawned
                determine_create_new_branches,
                apply_system_buffers, // this makes sure new branches are spawned
                assign_thicknesses,
                calculate_segment_lengths_and_tropism,
                update_branch_data_buffers,
            ).chain().after(step_physiological_age));

            if self.has_seeding {
                update_schedule.add_system((seed_plants).after(calculate_segment_lengths_and_tropism));
            }
    
        }
        else {world.spawn(MeshUpdateQueue::new(5));}



        /////// base graphics
        let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup(self.window_title.clone());
        let (swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
        let render_pass = if self.has_gui {double_pass_renderpass(&device, &swapchain)} else {single_pass_renderpass(&device, &swapchain)};
        let (framebuffers, window_dimensions) = get_framebuffers(&memory_allocator, &swapchain_images, &render_pass);


        let branch_mesh_settings = self.branch_render_settings.unwrap_or(DEFAULT_BRANCH_MESH_SETTINGS);
        add_world_branch_graphics_resources(&mut world, memory_allocator.clone(), branch_mesh_settings.0, branch_mesh_settings.1);
        

        /////// subpasses and pipelines
        

        let terrain_pipeline = match terrain_type {
            TerrainType::Absent => None,
            TerrainType::Bumpy => {
                create_terrain_mesh_buffers(&memory_allocator, &mut world);
                Some(get_heightmap_terrain_pipeline(window_dimensions, &device, &render_pass, 0))
            },
            TerrainType::Flat => {
                create_terrain_mesh_buffers(&memory_allocator, &mut world);
                Some(get_flat_terrain_pipeline(window_dimensions, &device, &render_pass, 0))
            }
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
        frame_schedule.add_system(update_all_meshes);


        // startrup
        let mut startup_schedule = Schedule::new();

        startup_schedule.add_system(init_branch_mesh_buffers_res);

        startup_schedule.run(&mut world);
        world.insert_resource(plant_species_sampler);
        world.insert_resource(MoistureAndTemp {
            moisture: environmental_params.2,
            temp_at_zero: environmental_params.0,
            temp_fall_off: environmental_params.1,
        });


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
            paused: false,

            
            graphics_pass,
            branch_pipeline,
            terrain: terrain_type,
            terrain_pipeline,
            terrain_settings: self.terrain_graphics_settings,
            gui,
            // this made me want to give up on the whole project - 2023-05-21: the second value was [0.0; 3] which caused no 3d graphics to render, I had been debugging for 3 days
            camera_state: ([-2.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            light: self.light.unwrap_or(DEFAULT_LIGHT)
        }
    }

    
}







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