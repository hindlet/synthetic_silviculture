#![allow(dead_code)]
use super::vector_two::Vector2;
use super::matrix_three::Matrix3;
use std::f32::consts::PI;
use std::ops::*;
use std::cmp::Ordering;

const HALF_PI: f32 = PI / 2.0;

#[derive(Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vector3 {
    pub x: f32, 
    pub y: f32,
    pub z: f32,
}

pub fn cross(lhs: Vector3, rhs: Vector3) -> Vector3{
    Vector3 {
        x: lhs.y * rhs.z - lhs.z * rhs.y,
        y: lhs.z * rhs.x - lhs.x * rhs.z,
        z: lhs.x * rhs.y - lhs.y * rhs.x,
    }
}


impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 {
            x,
            y,
            z,
        }
    }

    #[allow(non_snake_case)]
    pub const fn Y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0
        }
    }

    #[allow(non_snake_case)]
    pub const fn X() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    }

    #[allow(non_snake_case)]
    pub const fn Z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0
        }
    }
    
    #[allow(non_snake_case)]
    pub const fn ZERO() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[allow(non_snake_case)]
    pub const fn ONE() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }


    pub fn sqr_magnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn magnitude(&self) -> f32 {
        self.sqr_magnitude().sqrt()
    }

    pub fn normalise(&mut self){
        let length = self.magnitude();

        self.x /= length;
        self.y /= length;
        self.z /= length;
    }

    pub fn normalised(&self) -> Vector3 {
        let length = self.magnitude();

        Vector3::new(self.x / length, self.y / length, self.z / length)
    }

    pub fn dot(&self, rhs: impl Into<Vector3>) -> f32{
        let rhs: Vector3 = rhs.into();
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: impl Into<Vector3>) -> Vector3 {
        let rhs: Vector3 = rhs.into();
        Vector3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn outer_product(&self) -> [Vector3; 3]{
        [
            Vector3 {
                x: self.x * self.x,
                y: self.x * self.y,
                z: self.x * self.z,
            },
            Vector3 {
                x: self.y * self.x,
                y: self.y * self.y,
                z: self.y * self.z,
            },
            Vector3 {
                x: self.z * self.x,
                y: self.z * self.y,
                z: self.z * self.z,
            }
        ]
    }

    pub fn skew_symmetric(&self) -> [Vector3; 3] {
        [
            Vector3 {
                x: 0.0,
                y: -self.z,
                z: self.y,
            },
            Vector3 {
                x: self.z,
                y: 0.0,
                z: -self.x,
            },
            Vector3 {
                x: -self.y,
                y: self.x,
                z: 0.0,
            }
        ]
    }

    pub fn angle_to(&self, rhs: impl Into<Vector3>) -> f32 {
        let rhs: Vector3 = rhs.into();
        return (self.dot(rhs)/(self.magnitude() * rhs.magnitude())).acos();
    }


    pub fn transform(&self, transform: Matrix3) -> Vector3 {
        let x = self.dot(transform.x);
        let y = self.dot(transform.y);
        let z = self.dot(transform.z);
        Vector3::new(x, y, z)
    }

    pub fn mut_transform(&mut self, transform: Matrix3) {
        let new = self.transform(transform);
        self.x = new.x;
        self.y = new.y;
        self.z = new.z;
    }

    pub fn xy(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    pub fn xz(&self) -> Vector2 {
        Vector2::new(self.x, self.z)
    }

    pub fn yz(&self) -> Vector2 {
        Vector2::new(self.y, self.z)
    }

    /// gets the appropriate euler angles for a given direction vector, where (0, 0, 0)euler is equivivelant to (0, 1, 0)direction
    pub fn direction_to_euler_angles(start_dir: impl Into<Vector3>) -> Vector3{
        let mut start_dir: Vector3 = start_dir.into();
        start_dir.normalise();
        if start_dir == Vector3::Y() * -1.0 {
            Vector3::new(PI, 0.0, 0.0)
        } else if start_dir == Vector3::Y() {
            Vector3::ZERO()
        } else {
            let cross_mat = {
                let cross = start_dir.cross(Vector3::Y());
                Matrix3::new(
                    0.0, -cross.z, cross.y,
                    cross.z, 0.0, -cross.x,
                    -cross.y, cross.x, 0.0
                )
            };
            let angle_cos = start_dir.dot(Vector3::Y());
            let rot_mat =  Matrix3::identity() + cross_mat + cross_mat * cross_mat * (1.0 / (1.0 + angle_cos));
            Matrix3::euler_angles_from(rot_mat)
        }

    }

    /// gets the approriate direction vector for given euler angles, where (0, 1, 0)direction is equivelant to (0, 0, 0)euler
    pub fn euler_angles_to_direction(rot: impl Into<Vector3>) -> Vector3 {
        let matrix = Matrix3::from_euler_angles(rot);
        Vector3::Y().transform(matrix)
    }

    /// returns the directions of the different components of a direction vector: 
    /// 
    /// e.g: (-7.0, 5.0, -1.0) -> (-1, 1, -1)
    pub fn direction_directions(&self) -> Vector3 {
        Vector3::new(self.x / self.x.abs(), self.y / self.y.abs(), self.z / self.z.abs())
    }
}

//////////////////////////////////////////////////////////////////
///////////////////////////////// from and into
//////////////////////////////////////////////////////////////////

impl Into<[f32; 3]> for Vector3 {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}


impl From<[f32; 3]> for Vector3 {
    fn from(value: [f32; 3]) -> Self {
        Vector3::new(value[0], value[1], value[2])
    }
} 

impl From<[i32; 3]> for Vector3 {
    fn from(value: [i32; 3]) -> Self {
        Vector3::new(value[0] as f32, value[1] as f32, value[2] as f32)
    }
}

impl From<Vector2> for Vector3 {
    fn from(value: Vector2) -> Self {
        Vector3::new(value.x, value.y, 0.0)
    }
}
//////////////////////////////////////////////////////////////////
///////////////////////////////// arithmetic operations
//////////////////////////////////////////////////////////////////

impl Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<&Vector3> for Vector3 {
    type Output = Self;
    fn add(self, rhs: &Vector3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Div<Vector3> for Vector3 {
    type Output = Self;
    fn div(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
