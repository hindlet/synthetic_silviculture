#![allow(dead_code)]
use crate::vector_two::Vector2;
use crate::matrix_three::Matrix3;
use std::ops::*;
use std::cmp::Ordering;


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
    pub fn Y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0
        }
    }

    #[allow(non_snake_case)]
    pub fn X() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    }

    #[allow(non_snake_case)]
    pub fn Z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0
        }
    }
    
    #[allow(non_snake_case)]
    pub fn ZERO() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
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

    pub fn dot(&self, rhs: &Vector3) -> f32{
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Vector3) -> Vector3 {
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

    pub fn angle_to(&self, rhs: &Vector3) -> f32 {
        return (self.dot(rhs)/(self.magnitude() * rhs.magnitude())).acos();
    }

    pub fn transform(&mut self, transform: Matrix3) -> Self {
        let (x, y, z) = (self.x, self.y, self.z); 
        self.x = x * transform.x.x + y * transform.x.y + z * transform.x.z;
        self.y = x * transform.y.x + y * transform.y.y + z * transform.y.z;
        self.z = x * transform.z.x + y * transform.z.y + z * transform.z.z;
        *self
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
}

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

// arithmetic ops

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
