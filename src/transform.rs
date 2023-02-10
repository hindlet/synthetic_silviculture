#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::{
    vector_three::Vector3,
    vector_two::Vector2,
};

/// I used a vector 3 for rotation as I didn't want to implement quaternions
#[derive(Component)]
pub struct Transform {
    pub translation: Vector3,
    pub rotation: Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vector3::new(),
            rotation: Vector3::new(),
        }
    }
}


impl Transform {
    fn looking_at(&mut self, target: Vector3) {
        let angle_x = Vector2::dot(self.translation.yz(), target.yz()).acos();
        let angle_y = Vector2::dot(self.translation.xz(), target.xz()).acos();
        let angle_z = Vector2::dot(self.translation.xy(), target.xy()).acos();
    }
}