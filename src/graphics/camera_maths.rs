use bevy_ecs::prelude::Component;
use winit::event::VirtualKeyCode;
use crate::maths::{vector_three::{Vector3, cross}, matrix_three::Matrix3, matrix_four::Matrix4};


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
            position: Vector3::ZERO(),
            direction: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0
            },
            up: -Vector3::Y(),
            move_speed: 0.1,
            rotate_speed: 0.02,
            movement: [false; 10]
        }
    }
}

impl Camera {

    pub fn get_view_matrix(&self) -> Matrix4 {
        let f = {
            let mut f = self.direction.clone();
            f.normalise();
            f
        };
    
        let mut s = cross(self.up, f);

        s.normalise();
    
        let u = cross(f, s);
    
        Matrix4::new(
            s.x, u.x, -f.x, 0.0,
            s.y, u.y, -f.y, 0.0,
            s.z, u.z, -f.z, 0.0,
            -self.position.dot(&s), -self.position.dot(&u), self.position.dot(&f), 1.0
        )
    }

    pub fn look_at(&mut self, target: Vector3) {
        let mut dir = target - self.position;
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
        let mut left = cross(self.direction, self.up);

        let forward = cross(left, self.up);
        // forward/back
        if self.movement[0] {self.position -= forward * self.move_speed}
        if self.movement[1] {self.position += forward * self.move_speed}
        // left/right
        if self.movement[2] {self.position += left * self.move_speed}
        if self.movement[3] {self.position -= left * self.move_speed}
        // up/down
        if self.movement[4] {self.position -= self.up * self.move_speed}
        if self.movement[5] {self.position += self.up * self.move_speed}

        // spin around up
        // normalise up
        self.up.normalise();
        // outer product
        // rotate
        if self.movement[6] {
            let rotation = Matrix3::from_angle_and_axis(self.rotate_speed, self.up);
            self.direction.mut_transform(&rotation);
        }
        if self.movement[7] {
            let rotation = Matrix3::from_angle_and_axis(-self.rotate_speed, self.up);
            self.direction.mut_transform(&rotation);
        }

        // spin around left
        // normalise left
        left.normalise();
        // rotate
        if self.movement[8] {
            let rotation = Matrix3::from_angle_and_axis(self.rotate_speed, left);
            self.direction.mut_transform(&rotation);
        }
        if self.movement[9] {
            let rotation = Matrix3::from_angle_and_axis(-self.rotate_speed, left);
            self.direction.mut_transform(&rotation);
        }
    }
}

