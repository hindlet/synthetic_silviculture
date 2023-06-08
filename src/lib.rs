#![allow(unused_imports)]


mod maths;
mod plants;
mod branches;
#[cfg(feature = "vulkan_graphics")]
mod graphics;
mod environment;
mod debug;


pub mod apps;
pub use plants::plant::{PlantGrowthControlFactors, PlantPlasticityParameters};


