#![allow(dead_code)]
use crate::vector_three::Vector3;
use std::ops::*;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Matrix3 {
    pub x: Vector3, 
    pub y: Vector3,
    pub z: Vector3,
}

impl Matrix3 {
    pub fn new(
        c0r0: f32, c0r1: f32 , c0r2: f32,
        c1r0: f32, c1r1: f32 , c1r2: f32,
        c2r0: f32, c2r1: f32 , c2r2: f32,
    ) -> Self {
        Self {
            x: Vector3::new(c0r0, c0r1, c0r2),
            y: Vector3::new(c1r0, c1r1, c1r2),
            z: Vector3::new(c2r0, c2r1, c2r2),
        }
    }

    pub fn from_angle_y(angle: f32) -> Self{
        Matrix3::new(
            angle.cos(), 0.0, angle.sin(),
            0.0, 1.0, 0.0,
            -angle.sin(), 0.0, angle.cos(),
        )
    }


}