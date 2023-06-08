#![allow(dead_code, unused_variables, unused_imports)]
use std::{time::Instant, rc::Rc, cell::RefCell};
use itertools::Itertools;
use super::{
    super::maths::{vector_three::Vector3, bounding_sphere::BoundingSphere, matrix_three::Matrix3},
    branch_node::*,
};
#[cfg(feature = "vulkan_graphics")]
use super::super::graphics::mesh::Mesh;



///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// structs and components //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "vulkan_graphics")]
#[derive(Debug)]
pub struct Branch {
    pub data: BranchData,
    pub growth_data: BranchGrowthData,
    pub children: (Option<Rc<RefCell<Branch>>>, Option<Rc<RefCell<Branch>>>),

    pub root: Rc<RefCell<BranchNode>>,
    pub parent_node_index: usize, // the index of parent node in terminal nodes list of parent branch
    pub parent_index: usize, // the index of the parent branch in its layer 


    pub bounds: BoundingSphere,
    pub mesh: Mesh,
    pub needs_mesh_update: Option<Instant>,
    pub prototype_id: usize,
}

#[cfg(not(feature = "vulkan_graphics"))]
#[derive(Debug)]
pub struct Branch {
    pub data: BranchData,
    pub growth_data: BranchGrowthData,
    pub children: (Option<Rc<RefCell<Branch>>>, Option<Rc<RefCell<Branch>>>),

    pub root: Rc<RefCell<BranchNode>>,
    pub parent_node_index: usize, // the index of parent node in terminal nodes list of parent branch
    pub parent_index: usize, // the index of the parent branch in its layer 


    pub bounds: BoundingSphere,
    pub needs_mesh_update: Option<Instant>,
    pub prototype_id: usize,
}


#[derive(Debug)]
pub struct BranchData {
    pub intersections_volume: f32,
    pub normal: Vector3, // local positioning, global rotation
    pub root_position: Vector3, // global positioning, represents (0, 0, 0) in local coords
    pub finalised_mesh: bool,
}

#[derive(Debug)]
pub struct BranchGrowthData {
    pub light_exposure: f32,
    pub growth_vigor: f32,
    pub growth_rate: f32,
    pub physiological_age: f32,
    pub layers: u32,
}



///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Impl ///////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "vulkan_graphics")]
impl Branch {
    pub fn new(
        root_pos: Vector3,
        thickening_factor: f32,
        normal: Vector3,
        prototype_id: usize,
        parent_node_id: usize,
        parent_index: usize,
    ) -> Self {
        Branch {
            data: BranchData {root_position: root_pos, normal: normal, ..Default::default()},
            growth_data: BranchGrowthData::default(),
            children: (None, None),
            parent_node_index: parent_node_id,
            parent_index: parent_index,

            prototype_id: prototype_id,
            bounds: BoundingSphere::ZERO(),
            mesh: Mesh::empty(),
            needs_mesh_update: None,

            root: Rc::new(RefCell::new(BranchNode {
                children: Vec::new(),
                parent: 0,
                data: BranchNodeData {
                    relative_position: Vector3::ZERO(),
                    thickening_factor: thickening_factor,
                    ..Default::default()
                },
                growth_data: BranchNodeGrowthData::default()
            }))
        }
    }
}


#[cfg(not(feature = "vulkan_graphics"))]
impl Branch {
    pub fn new(
        root_pos: Vector3,
        thickening_factor: f32,
        normal: Vector3,
        prototype_id: usize,
        parent_node_id: usize,
        parent_index: usize,
    ) -> Self {
        Branch {
            data: BranchData {root_position: root_pos, normal: normal, ..Default::default()},
            growth_data: BranchGrowthData::default(),
            children: (None, None),
            parent_node_index: parent_node_id,
            parent_index: parent_index,

            prototype_id: prototype_id,
            bounds: BoundingSphere::ZERO(),
            needs_mesh_update: None,

            root: Rc::new(RefCell::new(BranchNode {
                children: Vec::new(),
                parent: 0,
                data: BranchNodeData {
                    relative_position: Vector3::ZERO(),
                    thickening_factor: thickening_factor,
                    ..Default::default()
                },
                growth_data: BranchNodeGrowthData::default()
            }))
        }
    }
}


impl Default for BranchData {
    fn default() -> Self {
        BranchData {
            intersections_volume: 0.0,
            normal: Vector3::Y(),
            root_position: Vector3::ZERO(),
            finalised_mesh: false,
        }
    }
}

impl Default for BranchGrowthData {
    fn default() -> Self {
        BranchGrowthData {
            growth_vigor: 0.0,
            growth_rate: 0.0,
            light_exposure: 0.0,
            physiological_age: 0.0,
            layers: 1,
        }
    }
}