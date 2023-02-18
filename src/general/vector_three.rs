#![allow(dead_code, unused_variables, unused_imports)]
use crate::vector_two::Vector2;


#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32, 
    pub y: f32,
    pub z: f32,
}


impl Vector3 {
    pub fn new() -> Self {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from(x: f32, y: f32, z: f32) -> Self {
        Vector3 {
            x,
            y,
            z,
        }
    }

    pub fn get_parts(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub fn up() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    pub fn subtract(&mut self, other: &Vector3){
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }

    pub fn subtract_to_new(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn add(&mut self, other: &Vector3){
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }

    pub fn add_to_new(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn multiply(&mut self, multiplier: f32){
        self.x *= multiplier;
        self.y *= multiplier;
        self.z *= multiplier;
    }

    pub fn multiply_to_new(&self, multiplier: f32) -> Vector3 {
        Vector3 {
            x: self.x * multiplier,
            y: self.y * multiplier,
            z: self.z * multiplier,
        }
    }

    pub fn get_sqr_magnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn get_magnitude(&self) -> f32 {
        self.get_sqr_magnitude().sqrt()
    }

    pub fn normalise(&mut self){
        let length = self.get_magnitude();

        self.x /= length;
        self.y /= length;
        self.z /= length;
    }

    pub fn dot(vector_one: Vector3, vector_two: Vector3) -> f32{
        vector_one.x * vector_two.x + vector_one.y * vector_two.y + vector_one.z * vector_two.z
    }

    pub fn cross(vector_one: Vector3, vector_two: Vector3) -> Vector3 {
        Vector3 {
            x: vector_one.y * vector_two.z - vector_one.z * vector_two.y,
            y: vector_one.z * vector_two.x - vector_one.x * vector_two.z,
            z: vector_one.x * vector_two.y - vector_one.y * vector_two.x,
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

    pub fn xy(&self) -> Vector2 {
        Vector2 {
            x: self.x,
            y: self.y
        }
    }

    pub fn xz(&self) -> Vector2 {
        Vector2 {
            x: self.x,
            y: self.z
        }
    }

    pub fn yz(&self) -> Vector2 {
        Vector2 {
            x: self.y,
            y: self.z
        }
    }
}