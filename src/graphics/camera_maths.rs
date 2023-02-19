use bevy_ecs::prelude::Component;
use cgmath::{Matrix4, Vector4};
use winit::event::VirtualKeyCode;
use crate::vector_three::Vector3;
use crate::vector_two::Vector2;


#[derive(Component)]
pub struct Camera {
    pub position: Vector3, 
    pub direction: Vector3,
    pub up: Vector3,
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub movement: [bool; 10], // forward, back, left, right, up, down, spin right, spin left, spin forward, spin backward
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vector3::new(),
            direction: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0
            },
            up: Vector3::up().multiply_to_new(-1.0),
            move_speed: 0.1,
            rotate_speed: 0.02,
            movement: [false; 10]
        }
    }
}

impl Camera {

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let f = {
            let mut f = self.direction.clone();
            f.normalise();
            f
        };
    
        let mut s = Vector3::cross(self.up, f);

        s.normalise();
    
        let u = Vector3::cross(f, s);
    

        Matrix4 {
            x: Vector4::new(s.x, u.x, -f.x, 0.0),
            y: Vector4::new(s.y, u.y, -f.y, 0.0),
            z: Vector4::new(s.z, u.z, -f.z, 0.0),
            w: Vector4::new(-self.position.dot(&s), -self.position.dot(&u), self.position.dot(&f), 1.0),
        }
    }

    pub fn look_at(&mut self, target: Vector3) {
        let mut dir = target.subtract_to_new(&self.position);
        dir.normalise();
        self.direction = dir;
    }

    pub fn process_key(&mut self, keycode: VirtualKeyCode, state: bool) {
        match keycode {
            VirtualKeyCode::W => {
                self.movement[0] = state;
            }
            VirtualKeyCode::S => {
                self.movement[1] = state;
            }
            VirtualKeyCode::A => {
                self.movement[2] = state;
            }
            VirtualKeyCode::D => {
                self.movement[3] = state;
            }
            VirtualKeyCode::Space => {
                self.movement[4] = state;
            }
            VirtualKeyCode::C => {
                self.movement[5] = state;
            }
            VirtualKeyCode::Q => {
                self.movement[6] = state;
            }
            VirtualKeyCode::E => {
                self.movement[7] = state;
            }
            VirtualKeyCode::R => {
                self.movement[8] = state;
            }
            VirtualKeyCode::F => {
                self.movement[9] = state;
            }
            _ => ()
        }
    }

    pub fn do_move(&mut self) {

        // take cross of direction and up to get left
        let mut left = Vector3::cross(self.direction, self.up);

        let forward = Vector3::cross(left, self.up);
        // forward/back
        if self.movement[0] {self.position.subtract(&forward.multiply_to_new(self.move_speed))}
        if self.movement[1] {self.position.add(&forward.multiply_to_new(self.move_speed))}
        // left/right
        if self.movement[2] {self.position.add(&left.multiply_to_new(self.move_speed))}
        if self.movement[3] {self.position.subtract(&left.multiply_to_new(self.move_speed))}
        // up/down
        if self.movement[4] {self.position.subtract(&self.up.multiply_to_new(self.move_speed))}
        if self.movement[5] {self.position.add(&self.up.multiply_to_new(self.move_speed))}

        // spin around up
        // normalise up
        self.up.normalise();
        // outer product
        // rotate
        if self.movement[6] {
            let spin_around_up = Vector3::get_rotate_matrix(self.up, self.rotate_speed);
            self.direction.rotate_vector(spin_around_up);
        }
        if self.movement[7] {
            let spin_around_up = Vector3::get_rotate_matrix(self.up, -self.rotate_speed);
            self.direction.rotate_vector(spin_around_up);
        }

        // spin around left
        // normalise left
        left.normalise();
        // rotate
        if self.movement[8] {
            let spin_around_left = Vector3::get_rotate_matrix(left, self.rotate_speed);
            self.direction.rotate_vector(spin_around_left);
        }
        if self.movement[9] {
            let spin_around_left = Vector3::get_rotate_matrix(left, -self.rotate_speed);
            self.direction.rotate_vector(spin_around_left);
        }
    }
}





impl Vector3 {

    fn get_rotate_matrix(axis: Vector3, angle: f32) -> [Vector3; 3] {
        let cos = angle.cos();
        let sin = angle.sin();
        [
            Vector3::from(
                cos + axis.x.powi(2) * (1.0 - cos),
                axis.x * axis.y * (1.0 - cos) - axis.z * sin,
                axis.x * axis.z * (1.0 - cos) + axis.y * sin,
            ),
            Vector3::from(
                axis.y * axis.x * (1.0 - cos) + axis.z * sin,
                cos + axis.y.powi(2) * (1.0 - cos),
                axis.y * axis.z * (1.0 - cos) - axis.x * sin,
            ),
            Vector3::from(
                axis.z * axis.x * (1.0 - cos) - axis.y * sin,
                axis.z * axis.y * (1.0 - cos) + axis.x * sin,
                cos + axis.z.powi(2) * (1.0 - cos),
            )
        ]
    }

    /// R = Cos(theta)I + sin(theta)Unit_Vector_Cross + (1-cos(theta))Outer_Product
    fn rotate_vector(&mut self, around: [Vector3; 3]) {
        self.x = self.x*around[0].x + self.y*around[0].y + self.z*around[0].z;
        self.y = self.x*around[1].x + self.y*around[1].y + self.z*around[1].z;
        self.z = self.x*around[2].x + self.y*around[2].y + self.z*around[2].z;
    }
}



