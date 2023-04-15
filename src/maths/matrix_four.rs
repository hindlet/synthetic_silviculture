#![allow(dead_code)]
use super::vector_four::Vector4;
use super::matrix_three::Matrix3;
use super::vector_three::Vector3;
use std::ops::*;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Matrix4 {
    pub x: Vector4, 
    pub y: Vector4,
    pub z: Vector4,
    pub w: Vector4,
}

impl Matrix4 {
    pub fn new(
        c0r0: f32, c0r1: f32, c0r2: f32, c0r3: f32,
        c1r0: f32, c1r1: f32, c1r2: f32, c1r3: f32,
        c2r0: f32, c2r1: f32, c2r2: f32, c2r3: f32,
        c3ro: f32, c3r1: f32, c3r2: f32, c3r3: f32,
    ) -> Self {
        Self {
            x: Vector4::new(c0r0, c0r1, c0r2, c0r3),
            y: Vector4::new(c1r0, c1r1, c1r2, c1r3),
            z: Vector4::new(c2r0, c2r1, c2r2, c2r3),
            w: Vector4::new(c3ro, c3r1, c3r2, c3r3)
        }
    }

    pub fn from_cols(x: Vector4, y: Vector4, z: Vector4, w: Vector4) -> Self {
        Self {x, y, z, w}
    }

    /// creates a perspective matrix for the specified settings, based on the opengl implementation
    pub fn persective_matrix(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> Self {
        let f = 1.0 / (fovy / 2.0).tan();
        Matrix4::new(
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (znear + zfar ) / (znear - zfar), -1.0,
            0.0, 0.0, (2.0 * zfar * znear) / (znear - zfar), 0.0
        )
    }

    // /// creates a view matrix for the given position and direction
    // pub fn view_matrix(view_dir: Vector3, view_pos: Vector3) -> Self {

    // }

}

impl Into<[[f32; 4]; 4]> for Matrix4 {
    fn into(self) -> [[f32; 4]; 4] {
        [self.x.into(), self.y.into(), self.z.into(), self.w.into()]
    }
}

impl From<Matrix3> for Matrix4 {
    fn from(mat: Matrix3) -> Self {
        Matrix4::new(
            mat.x.x, mat.x.y, mat.x.z, 0.0,
            mat.y.x, mat.y.y, mat.y.z, 0.0,
            mat.z.x, mat.z.y, mat.z.z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }
}


impl Mul for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: Self) -> Self::Output {
        let a = self.x;
        let b = self.y;
        let c = self.z;
        let d = self.w;

        Matrix4::from_cols(
            a*rhs.x.x + b*rhs.x.y + c*rhs.x.z + d*rhs.x.w,
            a*rhs.y.x + b*rhs.y.y + c*rhs.y.z + d*rhs.y.w,
            a*rhs.z.x + b*rhs.z.y + c*rhs.z.z + d*rhs.z.w,
            a*rhs.w.x + b*rhs.w.y + c*rhs.w.z + d*rhs.w.w,
        )
    }
}