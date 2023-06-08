use crate::graphics::mesh::Mesh;

use super::super::{
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
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::Window, platform::run_return::EventLoopExtRunReturn,
};
use core::time;
use std::{time::{Duration, Instant}, sync::Arc, cell::RefCell, rc::Rc, borrow::BorrowMut};
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

    gravity_strength: Option<f32>,
    time_step: Option<f32>,
    cell_settings: Option<(u32, f32)>,
    plant_death_rate: Option<f32>,
    environmental_params: Option<(f32, f32, f32)>, // temp at y=0, temp falloff, moisture

    prototype_conditions: Option<(Vec<(f32, f32)>, f32, f32)>,
    prototypes: Option<Vec<(f32, Vec<Vec<u32>>,  Vec<[f32; 3]>)>>,
    start_plants: u32,
    plant_species: Option<Vec<((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))>>,


}





pub struct GraphicsTreeApp {
    plants: Vec<Plant>,
    gravity: GravitySettings,
    time_step: f32,
    _terrain: Terrain,
    lightcells: LightCells,
    plant_death_rate: f32,
    branch_prototypes: BranchPrototypes,
    branch_sampler: BranchPrototypesSampler,

    // scheduling
    accumulated: Duration,
    period: Duration,


    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    event_loop: EventLoop<()>,
    memory_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,


    output: OutputType,

    graphics_pass: Subpass,
    light: ([f32; 3], f32),

    branch_pipeline: Arc<GraphicsPipeline>,
    branch_graphics_settings: BranchGraphicsSettings,
    branch_mesh_buffers: BranchMeshBuffers,

    terrain_type: TerrainType,
    terrain_pipeline: Arc<GraphicsPipeline>,
    terrain_graphics_settings: ([f32; 3], [f32; 3], f32, f32),
    terrain_mesh_buffers: TerrainMeshBuffers,

    gui: Option<Gui>,
    gui_data: Vec<GUIData>,

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
            prototypes: None,
            prototype_conditions: None,
            time_step: None,
            cell_settings: None,
            plant_death_rate: None,
            start_plants: 0,

            environmental_params: None,
            plant_species: None,
        }
    }


    pub fn run(mut self) -> TreeAppOutput{


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
                                _ => {output_ref.replace(TreeAppOutput{data: Some(data_output(&self.plants)), meshes: None});}
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
                    camera.do_move(delta_time);
                    self.accumulated += delta_time;
                    last_frame_time = Instant::now();
                    

                    // schedules
                    
                    if let Some(new_accum) = self.accumulated.checked_sub(self.period) {
                        self.accumulated = new_accum;

                        update_branch_bounds(&self.plants);
                        step_plant_age(&mut self.plants, self.time_step, self.plant_death_rate);
                        calculate_branch_light_exposure(&self.plants, &mut self.lightcells);
                        calculate_growth_vigor(&self.plants);
                        assign_growth_rates(&self.plants);
                        step_physiological_age(&self.plants, self.time_step);
                        update_branch_nodes(&self.plants, &self.branch_prototypes);
                        determine_create_new_branches(&self.plants, &self.branch_sampler, &self.branch_prototypes, &self.gravity);
                        assign_thicknesses(&self.plants);
                        calculate_segment_lengths_and_tropism(&self.plants, &self.branch_prototypes, &self.gravity);
                    }



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
                            match self.terrain_type {
                                TerrainType::Absent => {},
                                TerrainType::Flat => {terrain_pipeline = get_flat_terrain_pipeline(new_dimensions, &device, &render_pass, 0)},
                                TerrainType::Bumpy => {terrain_pipeline = get_heightmap_terrain_pipeline(new_dimensions, &device, &render_pass, 0)}
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
                    match self.terrain_type {
                        TerrainType::Absent => {},

                        TerrainType::Flat => {
                            let terrain_uniforms = create_flat_uniform_buffer(&swapchain, &camera, self.light, self.terrain_graphics_settings.0, &uniform_allocator);
                            add_flat_terrain_draw_commands(&mut secondary_builder, &terrain_pipeline, &descriptor_set_allocator, &terrain_uniforms, 0, &self.terrain_mesh_buffers);
                        },

                        TerrainType::Bumpy => {
                            let terrain_uniforms = create_heightmap_uniform_buffer(&swapchain, &camera, self.light, self.terrain_graphics_settings, &uniform_allocator);
                            add_heightmap_terrain_draw_commands(&mut secondary_builder, &terrain_pipeline, &descriptor_set_allocator, &terrain_uniforms, 0, &self.terrain_mesh_buffers);
                        }
                    }

                    ////// branch_graphics
                    update_next_mesh(&self.plants, &self.branch_graphics_settings);
                    update_branch_data_buffers(&self.plants, &self.branch_graphics_settings, &mut self.branch_mesh_buffers, &self.memory_allocator);
                    let branch_uniforms = create_branch_uniform_buffer(&swapchain, &camera, self.light, &uniform_allocator);
                    add_branch_draw_commands(&mut secondary_builder, &branch_pipeline, &descriptor_set_allocator, &branch_uniforms, &self.branch_mesh_buffers);
                    

                    builder.execute_commands(secondary_builder.build().unwrap()).unwrap();

                    ////// gui graphics
                    if let Some(gui) = self.gui.as_mut() {
                        draw_gui(&mut self.gui_data, gui);
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
    pub fn set_plant_species(&mut self, species: Vec<((PlantGrowthControlFactors, PlantPlasticityParameters), (f32, f32, f32, f32))>) -> &mut GraphicsAppBuilder {
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




    /// builds and returns the app for running
    pub fn build(&mut self) -> GraphicsTreeApp{

        ///////////////// transform data
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
        let has_plants = self.start_plants > 0;
        let plant_species = self.plant_species.clone().unwrap_or(DEFAULT_PLANT_SPECIES);
        let environmental_params = self.environmental_params.unwrap_or(DEFAULT_ENVIRONMENTAL_PARAMS);
        
        


        ///////////////// create data structs
        let branch_sampler = BranchPrototypesSampler::create(branch_conditions.0, SAMPLER_SIZE, branch_conditions.1, branch_conditions.2);
        let plant_species_sampler = PlantSpeciesSampler::new(plant_species);

        let gravity = GravitySettings::create([0, -1, 0], gravity_strength);
        let prototypes = BranchPrototypes::new(branch_types);
        let cells = LightCells::new(cell_settings.0 as i32, cell_settings.1);


        let (terrain_type, plant_spawning_bounds, terrain) = {
            if self.has_terrain {     
                let settings = self.terrain_settings.clone().unwrap();
                if settings.2.is_none() {
                    let (terrain, bounds) = create_flat_terrain(settings.0, settings.1);
                    (TerrainType::Flat, bounds, terrain)
                }
                else {
                    let subsettings = settings.2.clone().unwrap();
                    let (terrain, bounds) = create_heightmap_terrain(settings.0, subsettings.0, subsettings.1, settings.1, subsettings.2);
                    (TerrainType::Bumpy, bounds, terrain)
                }
            }
            else {
                let (terrain, bounds) = create_flat_terrain(DEFAULT_TERRAIN.0, DEFAULT_TERRAIN.1);
                (TerrainType::Absent, bounds, terrain)
            }
        };

        //////////// spawn initial plant(s)

        let mut plants = Vec::new();
        let mut rng = thread_rng();

        for _i in 0..self.start_plants {

            let (x, z) = (rng.gen_range(plant_spawning_bounds.1.clone()), rng.gen_range(plant_spawning_bounds.2.clone()));

            let hit_pos = terrain.check_ray([x, plant_spawning_bounds.0 + 5.0, z], [0, -1, 0], None).unwrap();
            let spawn_data = plant_species_sampler.get_plant(environmental_params.0 + hit_pos.hit_position.y * environmental_params.1, environmental_params.2);

            if let Some((growth_factors, plasticity)) = spawn_data {
                let apical = growth_factors.apical_control;
                plants.push(
                    Plant::new(
                        plasticity,
                        growth_factors,
                        [x, plant_spawning_bounds.0 + 5.0, z],
                        hit_pos.hit_normal,
                        branch_sampler.get_prototype_index(apical, branch_conditions.2), // max determinancy
                    )
                )
            }
        }
        
        if has_plants &&  plants.len() == 0{panic!("No intial plants generated")}



        /////// base graphics
        let (queue, device, physical_device, surface, event_loop, memory_allocator) = base_graphics_setup(self.window_title.clone());
        let (swapchain, swapchain_images) = get_swapchain(&physical_device, &surface, &device);
        let render_pass = if self.has_gui {double_pass_renderpass(&device, &swapchain)} else {single_pass_renderpass(&device, &swapchain)};
        let (framebuffers, window_dimensions) = get_framebuffers(&memory_allocator, &swapchain_images, &render_pass);

        

        /////// subpasses and pipelines
        

        let (terrain_pipeline, terrain_mesh_buffers) = match terrain_type {
            TerrainType::Absent => {
                let buffers = create_terrain_mesh_buffers(&memory_allocator, &Mesh::empty());
                (GraphicsPipeline::start().build(device.clone()).unwrap(), buffers)
            },
            TerrainType::Bumpy => {
                let buffers = create_terrain_mesh_buffers(&memory_allocator, &terrain.mesh);
                (get_heightmap_terrain_pipeline(window_dimensions, &device, &render_pass, 0), buffers)
            },
            TerrainType::Flat => {
                let buffers = create_terrain_mesh_buffers(&memory_allocator, &terrain.mesh);
                (get_flat_terrain_pipeline(window_dimensions, &device, &render_pass, 0), buffers)
            }
        };

        let branch_pipeline = get_branch_pipeline(window_dimensions, &device, &render_pass, 0);
        let branch_graphics_settings = create_branch_graphics_settings(3, false);
        let branch_mesh_buffers = init_branch_mesh_buffers(&plants, &branch_graphics_settings, &memory_allocator);

        let graphics_pass = Subpass::from(render_pass.clone(), 0).unwrap();

        let (gui, gui_data) = {
            if self.has_gui {
                let subpass = Subpass::from(render_pass.clone(), 1).unwrap();
               (Some(create_gui_from_subpass(&event_loop, &surface, &queue, &subpass)), Vec::new())
            }
            else {(None, Vec::new())}
        };


        GraphicsTreeApp{

            plants: plants,
            gravity: gravity,
            time_step: time_step,
            _terrain: terrain,
            lightcells: cells,
            plant_death_rate: plant_death_rate,
            branch_prototypes: prototypes,
            branch_sampler: branch_sampler,

            accumulated: Duration::ZERO,
            period: Duration::from_secs_f32(0.1),

            device,
            queue,
            surface,
            event_loop,
            memory_allocator,
            swapchain,
            render_pass,
            framebuffers,

            output,

            graphics_pass,
            light: self.light.unwrap_or(DEFAULT_LIGHT),

            branch_pipeline,
            branch_graphics_settings: branch_graphics_settings,
            branch_mesh_buffers: branch_mesh_buffers,

            terrain_type: terrain_type,
            terrain_pipeline,
            terrain_graphics_settings: self.terrain_graphics_settings.unwrap_or(DEFAULT_TERRAIN_SETTINGS),
            terrain_mesh_buffers: terrain_mesh_buffers,

            gui,
            gui_data: gui_data,
            // this made me want to give up on the whole project - 2023-05-21: the second value was [0.0; 3] which caused no 3d graphics to render, I had been debugging for 3 days
            camera_state: ([-2.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            
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