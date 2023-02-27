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

/// creates a rotation maxtrix for anticlockwise angle around y axis
    pub fn from_angle_y(angle: f32) -> Self{
        Matrix3::new(
            angle.cos(), 0.0, angle.sin(),
            0.0, 1.0, 0.0,
            -angle.sin(), 0.0, angle.cos()
        )
    }

/// creates a rotation maxtrix for anticlockwise angle around x axis
    pub fn from_angle_x(angle: f32) -> Self {
        Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, angle.cos(), -angle.sin(),
            0.0, angle.sin(), angle.cos()
        )
    }

    /// creates a rotation maxtrix for anticlockwise angle around z axis
    pub fn from_angle_z(angle: f32) -> Self {
        Matrix3::new(
            angle.cos(), -angle.sin(), 0.0,
            angle.sin(), angle.cos(), 0.0,
            0.0, 0.0, 1.0,
        )
    }

    /// creates a rotation matrix for anticlockwise rotation of angle around the specified axis
    pub fn from_angle_and_axis(angle: f32, axis: Vector3) -> Self {
        Matrix3::new(
            angle.cos() + axis.x.powi(2) * (1.0 - angle.cos()),
            axis.x * axis.y * (1.0 - angle.cos()) - axis.z * angle.sin(),
            axis.x * axis.z * (1.0 - angle.cos()) + axis.y * angle.sin(),
            axis.y * axis.x * (1.0 - angle.cos()) + axis.z * angle.sin(),
            angle.cos() + axis.y.powi(2) * (1.0 - angle.cos()),
            axis.y * axis.z * (1.0 - angle.cos()) - axis.x * angle.sin(),
            axis.z * axis.x * (1.0 - angle.cos()) - axis.y * angle.sin(),
            axis.z * axis.y * (1.0 - angle.cos()) + axis.x * angle.sin(),
            angle.cos() + axis.z.powi(2) * (1.0 - angle.cos()),
        )
    }

    // creates a transform matrix for scaling by specied multiplier
    pub fn from_scale(scale: f32) -> Self{
        Matrix3::new(
            scale, 0.0, 0.0,
            0.0, scale, 0.0,
            0.0, 0.0, scale,
        )
    }


}