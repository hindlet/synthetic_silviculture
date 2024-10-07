use std::collections::HashMap;

use maths::{BoundingSphere, Vector3};


pub struct BranchPrototype {
    pub node_positions: Vec<Vector3>,
    pub node_connections: Vec<(usize, usize)>,
    pub terminal_nodes: Vec<usize> // max of 2
}


pub struct BranchNode<'node_lifetime> {
    pub position: Vector3,
    pub phys_age: f32,
    pub branch_length: f32,
    pub thickening_factor: f32,

    pub parent: Box<&'node_lifetime BranchNode<'node_lifetime>>

}

pub struct Branch<'branch_lifetime, 'node_lifetime> {
    pub nodes: HashMap<usize, BranchNode<'node_lifetime>>,
    pub bounding_volume: BoundingSphere,

    pub intersection_volume: f32,
    pub light_sum_at_point: f32,

    pub parent: Box<&'branch_lifetime Branch<'branch_lifetime, 'node_lifetime>>,
    pub child_one: Option<Box<&'branch_lifetime Branch<'branch_lifetime, 'node_lifetime>>>,
    pub child_two: Option<Box<&'branch_lifetime Branch<'branch_lifetime, 'node_lifetime>>>
}

impl<'branch_lifetime, 'node_lifetime> Branch<'branch_lifetime, 'node_lifetime> {

    /// calculate and assign the bounding volume to the branch
    pub fn calculate_bounding_volume(&mut self) {
        self.bounding_volume = BoundingSphere::from_points(self.nodes.iter().map(|(_, n)| n.position).collect())
    }





}