#![allow(dead_code)]
use super::vector_three::Vector3;
use std::{ops::*, f32::consts::PI};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Matrix3 {
    pub x: Vector3, 
    pub y: Vector3,
    pub z: Vector3,
}

impl Matrix3 {
    pub fn new(
        r0c0: f32, r0c1: f32 , r0c2: f32,
        r1c0: f32, r1c1: f32 , r1c2: f32,
        r2c0: f32, r2c1: f32 , r2c2: f32,
    ) -> Self {
        Matrix3 {
            x: Vector3::new(r0c0, r0c1, r0c2),
            y: Vector3::new(r1c0, r1c1, r1c2),
            z: Vector3::new(r2c0, r2c1, r2c2),
        }
    }

    pub fn from_rows(
        r0: Vector3,
        r1: Vector3,
        r2: Vector3,
    ) -> Self {
        Matrix3 {
            x: r0,
            y: r1,
            z: r2
        }
    }

    pub fn from_columns(
        c0: Vector3,
        c1: Vector3,
        c2: Vector3,
    ) -> Self {
        let mat = Matrix3::from_rows(c0, c1, c2);
        mat.transpose()
    }

    pub fn identity() -> Self {
        Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
        )
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
            0.0, 0.0, 1.0
        )
    }

    /// creates a rotation matrix for anticlockwise rotation of angle around the specified axis
    pub fn from_angle_and_axis(angle: f32, mut axis: Vector3) -> Self {
        axis.normalise();
        if angle == 0.0 {return Matrix3::identity();}
        Matrix3::new(
            angle.cos() + axis.x.powi(2) * (1.0 - angle.cos()),
            axis.x * axis.y * (1.0 - angle.cos()) - axis.z * angle.sin(),
            axis.x * axis.z * (1.0 - angle.cos()) + axis.y * angle.sin(),

            axis.y * axis.x * (1.0 - angle.cos()) + axis.z * angle.sin(),
            angle.cos() + axis.y.powi(2) * (1.0 - angle.cos()),
            axis.y * axis.z * (1.0 - angle.cos()) - axis.x * angle.sin(),

            axis.z * axis.x * (1.0 - angle.cos()) - axis.y * angle.sin(),
            axis.z * axis.y * (1.0 - angle.cos()) + axis.x * angle.sin(),
            angle.cos() + axis.z.powi(2) * (1.0 - angle.cos())
        )
    }

    pub fn from_euler_angles(angles: &Vector3) -> Matrix3 {
        let x = angles.x;
        let y = angles.y;
        let z = angles.z;
        Matrix3::new(
            y.cos() * z.cos(),
            x.sin() * y.sin() * z.cos() - x.cos() * z.sin(),
            x.cos() * y.sin() * z.cos() + x.sin() * z.sin(),

            y.cos() * z.sin(),
            x.sin() * y.sin() * z.sin() + x.cos() * z.cos(),
            x.cos() * y.sin() * z.sin() - x.sin() * z.cos(),

            -(y.sin()),
            x.sin() * y.cos(),
            x.cos() * y.cos()
        )
    }

    // calculates the euler angles required to create a specific matrix
    pub fn euler_angles_from(rot: &Matrix3) -> Vector3 {
        let mut angles = Vector3::ZERO();

        // special cases
        if rot.z.x == 1.0{
            angles.y = -PI / 2.0;
            angles.x = -(rot.x.y).atan2(-rot.x.z);
            return angles;
        }
        if rot.z.x == -1.0 {
            angles.y = PI / 2.0;
            angles.x = rot.x.y.atan2(rot.x.z);
            return angles;
        }

        // get y angle
        angles.y = -rot.z.x.asin();

        // get x angle
        angles.x = (rot.z.y / angles.y.cos()).atan2(rot.z.z / angles.y.cos());

        // get z angle
        angles.z = (rot.y.x / angles.y.cos()).atan2(rot.x.x / angles.y.cos());

        angles
    }   


    // creates a transform matrix for scaling by specied multiplier
    pub fn from_scale(scale: f32) -> Self{
        Matrix3::new(
            scale, 0.0, 0.0,
            0.0, scale, 0.0,
            0.0, 0.0, scale
        )
    }

    pub fn transpose(&self) -> Matrix3{
        Matrix3::new(
            self.x.x, self.y.x, self.z.x,
            self.x.y, self.y.y, self.z.y,
            self.x.z, self.y.z, self.z.z
        )
    }

    pub fn transpose_self(&mut self) {

    }

    // returns the determinant of a given matrix
    pub fn determinant(&self) -> f32 {
        self.x.x * (self.y.y * self.z.z - self.z.y * self.y.z)
            - self.x.y * (self.y.x * self.z.z - self.z.x * self.y.z)
            + self.x.z * (self.y.x * self.z.y - self.z.x * self.y.y)
    }

    // returns the inverse the given matrix, equivelent to matrix^-1
    pub fn inverted(&self) -> Matrix3{
        let det = self.determinant();
        if det == 0.0 {return self.clone();}

        let c0  = self.y.cross(&self.z);
        let c1  = self.z.cross(&self.x);
        let c2  = self.x.cross(&self.y);
        Matrix3::from_columns(c0, c1, c2) / det
    }
}



impl Mul for Matrix3 {
    type Output = Matrix3;
    fn mul(self, rhs: Self) -> Self::Output {

        Matrix3::new(
            self.x.x * rhs.x.x + self.x.y * rhs.y.x + self.x.z * rhs.z.x,
            self.x.x * rhs.x.y + self.x.y * rhs.y.y + self.x.z * rhs.z.y,
            self.x.x * rhs.x.z + self.x.y * rhs.y.z + self.x.z * rhs.z.z,

            self.y.x * rhs.x.x + self.y.y * rhs.y.x + self.y.z * rhs.z.x,
            self.y.x * rhs.x.y + self.y.y * rhs.y.y + self.y.z * rhs.z.y,
            self.y.x * rhs.x.z + self.y.y * rhs.y.z + self.y.z * rhs.z.z,

            self.z.x * rhs.x.x + self.z.y * rhs.y.x + self.z.z * rhs.z.x,
            self.z.x * rhs.x.y + self.z.y * rhs.y.y + self.z.z * rhs.z.y,
            self.z.x * rhs.x.z + self.z.y * rhs.y.z + self.z.z * rhs.z.z,
        )
    }
}

impl Add for Matrix3 {
    type Output = Matrix3;
    fn add(self, rhs: Self) -> Self::Output {
        Matrix3::from_rows(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z
        )
    }
}

impl Mul<f32> for Matrix3 {
    type Output = Matrix3;
    fn mul(self, rhs: f32) -> Self::Output {
        Matrix3::from_rows(
            self.x * rhs,
            self.y * rhs,
            self.z * rhs,
        )
    }
}

impl Div<f32> for Matrix3 {
    type Output = Matrix3;
    fn div(self, rhs: f32) -> Self::Output {
        Matrix3::from_rows(
            self.x / rhs,
            self.y / rhs,
            self.z / rhs,
        )
    }
}

