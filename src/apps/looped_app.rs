use super::{*, super::{
    maths::vector_three::Vector3,
}};



pub struct LoopedTreeApp {
    output: OutputType,

    // terrain_settings: (f32, Vector3, Option<(u32, f32, String)>), // size, centre, verts per side, height mult, path
    gravity_strength: f32,
    time_step: f32,
    cell_settings: (u32, f32),
    plant_death_rate: f32,
}

impl Default for LoopedTreeApp {
    fn default() -> Self {
        LoopedTreeApp {
            output: OutputType::Absent,

            // terrain_settings: DEFAULT_TERRAIN,
            gravity_strength: DEFAULT_GRAVITY_STRENGTH,
            time_step: DEFAULT_TIMESTEP,
            cell_settings: DEFAULT_CELL_SETTINGS,
            plant_death_rate: DEFAULT_PLANT_DEATH_RATE,
        }
    }
}


impl LoopedTreeApp {
    pub fn new() -> Self {
        LoopedTreeApp::default()
    }

    pub fn reset(&mut self) -> &mut LoopedTreeApp {
        self.output = OutputType::Absent;

        // self.terrain_settings = DEFAULT_TERRAIN;
        self.gravity_strength = DEFAULT_GRAVITY_STRENGTH;
        self.time_step = DEFAULT_TIMESTEP;
        self.cell_settings = DEFAULT_CELL_SETTINGS;
        self.plant_death_rate = DEFAULT_PLANT_DEATH_RATE;

        self
    }
}