#![allow(dead_code, unused_variables, unused_imports)]
use std::mem::take;
use super::super::maths::vector_three::Vector3;


#[derive(Debug)]
pub struct BranchNode {
    pub data: BranchNodeData,
    pub growth_data: BranchNodeGrowthData,
    pub children: Vec<BranchNode>,
    pub parent: usize // index of parent node in its layer
}


#[derive(Debug)]
pub struct BranchNodeData {
    pub relative_position: Vector3, // local positioning, global rotation
    pub phys_age: f32,
    pub radius: f32,
    pub thickening_factor: f32,
}

#[derive(Debug)]
pub struct BranchNodeGrowthData {
    pub light_exposure: f32,
    pub growth_vigor: f32,
}

impl Default for BranchNode {
    fn default() -> Self {
        BranchNode{
            data: BranchNodeData::default(),
            growth_data: BranchNodeGrowthData::default(),
            children: Vec::new(),
            parent: 0
        }
    }
}


impl Default for BranchNodeData {
    fn default() -> Self {
        BranchNodeData {
            relative_position: Vector3::ZERO(),
            phys_age: 0.0,
            radius: 0.0,
            thickening_factor: 0.0,
        }
    }
}

impl Default for BranchNodeGrowthData {
    fn default() -> Self {
        BranchNodeGrowthData {
            light_exposure: 0.0,
            growth_vigor: 0.0,
        }
    }
}