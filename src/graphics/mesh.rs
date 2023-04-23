//! generic code for meshes in 3D space


use bevy_ecs::prelude::*;
use crate::maths::vector_three::Vector3;
use itertools::Itertools;
use super::general_graphics::{PositionVertex, Normal};


/// Data for a mesh in 3D space
#[derive(Debug, Component, Clone)]
pub struct Mesh {
    pub vertices: Vec<PositionVertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<u32>,
}


impl Mesh {
    pub fn empty() -> Self {
        Mesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new()
        }
    }

    pub fn new(
        vertices: Vec<PositionVertex>,
        normals: Vec<Normal>,
        indices: Vec<u32>,
    ) -> Self {
        Mesh {
            vertices, normals, indices
        }
    }

    pub fn set(&mut self, new: Mesh) {
        self.vertices = new.vertices;
        self.normals = new.normals;
        self.indices = new.indices;
    }

    /// returns a flat shaded version of the mesh called on
    pub fn flat_shaded(&self) -> Mesh {
        let mut new_verts: Vec<PositionVertex> = Vec::new();
        let mut new_normals: Vec<Normal> = Vec::new();
    
        for i in (0..self.indices.len()).step_by(3) {
            let v_one: Vector3 = self.vertices[self.indices[i as usize + 0] as usize].into();
            let v_two: Vector3 = self.vertices[self.indices[i as usize + 1] as usize].into();
            let v_thr: Vector3 = self.vertices[self.indices[i as usize + 2] as usize].into();
    
            let normal = (v_two - v_one).cross(&(v_thr - v_one));
    
            new_verts.push(PositionVertex::from(v_one));
            new_verts.push(PositionVertex::from(v_two));
            new_verts.push(PositionVertex::from(v_thr));
            new_normals.push(Normal::from(normal));
            new_normals.push(Normal::from(normal));
            new_normals.push(Normal::from(normal));
        }
    
        let indices = (0..(new_verts.len()) as u32).collect_vec();
        Mesh::new(new_verts, new_normals, indices)
    }

    /// sets the current mesh to be flat shaded
    /// 
    /// NOT CURRENTLY REVERSIBLE
    pub fn flat_shade(&mut self) {
        let new = self.flat_shaded();
        self.set(new);
    }

    /// flat shades the components of a Mesh without ever needing a Mesh
    /// 
    /// functionally equivalent to calling flat_shaded() and then into()
    pub fn flat_shade_components(in_verts: Vec<PositionVertex>, in_inds: Vec<u32>) -> (Vec<PositionVertex>, Vec<Normal>, Vec<u32>){
        let mut new_verts: Vec<PositionVertex> = Vec::new();
        let mut new_normals: Vec<Normal> = Vec::new();
    
        for i in (0..in_inds.len()).step_by(3) {
            let v_one: Vector3 = in_verts[in_inds[i as usize + 0] as usize].into();
            let v_two: Vector3 = in_verts[in_inds[i as usize + 1] as usize].into();
            let v_thr: Vector3 = in_verts[in_inds[i as usize + 2] as usize].into();
    
            let normal = (v_two - v_one).cross(&(v_thr - v_one));
    
            new_verts.push(PositionVertex::from(v_one));
            new_verts.push(PositionVertex::from(v_two));
            new_verts.push(PositionVertex::from(v_thr));
            new_normals.push(Normal::from(normal));
            new_normals.push(Normal::from(normal));
            new_normals.push(Normal::from(normal));
        }
    
        let indices = (0..(new_verts.len()) as u32).collect_vec();
        (new_verts, new_normals, indices)
    }
}

impl Into<(Vec<PositionVertex>, Vec<Normal>, Vec<u32>)> for Mesh{
    fn into(self) -> (Vec<PositionVertex>, Vec<Normal>, Vec<u32>) {
        (self.vertices, self.normals, self.indices)
    }
}