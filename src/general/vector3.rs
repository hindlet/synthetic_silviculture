use bevy_ecs::prelude::*;



#[derive(Default, Component, Clone, Copy, Debug, PartialEq)]
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
}