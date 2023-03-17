use egui::{self, Ui, Slider, Window, Frame, Color32, Context, FontFamily, FontDefinitions, FontData, epaint::Shadow, Stroke, Label};
use egui_winit_vulkano::{Gui, GuiConfig};
use vulkano::{swapchain::Surface, device::Queue, render_pass::{Subpass, RenderPass}, image::SampleCount, command_buffer::SecondaryAutoCommandBuffer};
use winit::{event_loop::EventLoop, event::WindowEvent};
use std::{sync::Arc, ops::{RangeInclusive, DerefMut}, borrow::Cow};
use vulkano_util::{
    window::VulkanoWindows
};
use bevy_ecs::prelude::*;
/////////////// structs

// my gui will only really be used to control settings so only needs to store a reference of what the settings are and what they effect
#[derive(Component)]
pub struct GUIData {
    pub name: String,
    pub bools: Vec<(String, bool)>,
    pub f32_sliders: Vec<(String, f32, RangeInclusive<f32>)>,
    pub i32_sliders: Vec<(String, i32, RangeInclusive<i32>)>,
}

impl GUIData {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // loop through checkboxes
            for checkbox in self.bools.iter_mut() {
                ui.checkbox(&mut checkbox.1, checkbox.0.clone());
            }

            ui.separator();

            for slider in self.f32_sliders.iter_mut() {
                ui.add(Label::new(slider.0.clone()));
                ui.add(Slider::new(&mut slider.1, slider.2.clone()));
            }

            for slider in self.i32_sliders.iter_mut() {
                ui.add(Label::new(slider.0.clone()));
                ui.add(Slider::new(&mut slider.1, slider.2.clone()));
            }

        });
    }
}

impl Default for GUIData {
    fn default() -> Self {
        GUIData {
            name: "".to_string(),
            bools: Vec::new(),
            f32_sliders: Vec::new(),
            i32_sliders: Vec::new(),
        }
    }
}


/////////////// functions

/// this just sets the gui style to my preffered
fn set_gui_style(
    ctx: &Context
) {
    let mut style: egui::Style = (*ctx.style()).clone();

    style.visuals.override_text_color = Some(Color32::from_rgb(250, 250, 250));

    style.visuals.widgets.inactive.bg_stroke = Stroke {
        width: 0.5,
        color: Color32::from_rgb(0, 0, 0)
    };

    style.visuals.button_frame = true;

    style.visuals.collapsing_header_frame = true;

    style.visuals.window_shadow = Shadow::NONE;

    style.visuals.window_fill = Color32::from_rgb(150, 150, 150);
    

    ctx.set_style(style);

    let font_droidsansmono = include_bytes!("DroidSansMono.ttf");
    let mut font = FontDefinitions::default();

    font.font_data.insert(
        "Droid Sans Mono".to_string(),
        FontData::from_static(font_droidsansmono),
    );

    font.families
        .insert(FontFamily::Proportional, vec!["Droid Sans Mono".to_string()]);

    ctx.set_fonts(font); 
}

pub fn create_gui_subpass(
    event_loop: &EventLoop<()>,
    surface: &Arc<Surface>,
    queue: &Arc<Queue>,
    render_pass: &Arc<RenderPass>,
) -> Gui{
    let gui = Gui::new_with_subpass(
        event_loop,
        surface.clone(),
        queue.clone(),
        Subpass::from(render_pass.clone(), 0).unwrap(),
        GuiConfig {
            preferred_format: Some(vulkano::format::Format::B8G8R8A8_SRGB),
            // Must match your pipeline's sample count
            samples: SampleCount::Sample4,
            ..Default::default()
        },
    );
    set_gui_style(&gui.context());
    gui
}

pub fn create_gui_from_subpass(
    event_loop: &EventLoop<()>,
    surface: &Arc<Surface>,
    queue: &Arc<Queue>,
    subpass: &Subpass,
) -> Gui{
    let gui = Gui::new_with_subpass(
        event_loop,
        surface.clone(),
        queue.clone(),
        subpass.clone(),
        GuiConfig {
            preferred_format: Some(vulkano::format::Format::B8G8R8A8_SRGB),
            // Must match your pipeline's sample count
            samples: SampleCount::Sample4,
            ..Default::default()
        },
    );
    set_gui_style(&gui.context());
    gui
}

pub fn get_gui_commands(
    gui: &mut Gui,
    dimensions: [u32; 2]
) -> SecondaryAutoCommandBuffer {
    gui.draw_on_subpass_image(dimensions)
}

/////////////// systems
/// 
#[derive(Resource)]
pub struct GUIResources {
    pub gui: Gui,
}


pub fn add_world_gui_resources(
    world: &mut World,
    gui: Gui,
) {
    world.insert_resource(GUIResources {
        gui
    })
}

pub fn pass_winit_event_to_gui(
    gui_res: Option<Mut<GUIResources>>,
    event: &WindowEvent
) -> bool {
    gui_res.unwrap().gui.update(event)
}

pub fn get_gui_resource_commands(
    gui_res: Option<Mut<GUIResources>>,
    dimensions: [u32; 2],
) -> SecondaryAutoCommandBuffer {
    get_gui_commands(&mut gui_res.unwrap().gui, dimensions)
}

pub fn draw_gui_objects(
    mut gui_query: Query<&mut GUIData>,
    mut gui_res: ResMut<GUIResources>
) {
    gui_res.deref_mut().gui.immediate_ui(|gui| {
        let ctx = gui.context();
        for mut gui_object in gui_query.iter_mut() {
            Window::new(gui_object.name.clone())
                .default_width(150.0)
                .show(&ctx, |ui| {
                    gui_object.ui(ui);
                });
        }
    });
}