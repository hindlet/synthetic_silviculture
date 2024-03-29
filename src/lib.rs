#![allow(unused_imports)]


mod maths;
mod plants;
mod branches;

mod fixed_schedule;
mod environment;
mod debug;


pub mod apps;
pub use plants::plant::{GrowthControlSettingParams, PlasticitySettingParams};

#[cfg(feature = "vulkan_graphics")]
mod graphics;