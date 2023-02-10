use bevy_ecs::prelude::Component;
use crate::vector_three::Vector3;

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
            direction: Vector3::new(),
            up: Vector3::UP(),
            move_speed: 0.5 * 0.0166667,
            rotate_speed: 0.02,
            movement: [false; 10]
        }
    }
}

impl Camera {

    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        let f = {
            let mut f = self.direction.clone();
            f.normalise();
            f
        };
    
        let mut s = Vector3::cross(self.up, f);

        s.normalise();
    
        let u = Vector3::cross(f, s);
    
        let p = Vector3 {
            x: -self.position.x * s.x- self.position.y * s.y - self.position.z * s.z,
            y: -self.position.x * u.x - self.position.y * u.y - self.position.z * u.z,
            z: -self.position.x * f.x - self.position.y * f.y - self.position.z * f.z
        };
    
        [
            [s.x, u.x, f.x, 0.0],
            [s.y, u.y, f.y, 0.0],
            [s.z, u.z, f.z, 0.0],
            [p.x, p.y, p.z, 1.0],
        ]
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
        if self.movement[4] {self.position.add(&self.up.multiply_to_new(self.move_speed))}
        if self.movement[5] {self.position.subtract(&self.up.multiply_to_new(self.move_speed))}

        // spin around up
        // normalise up
        self.up.normalise();
        // outer product
        let up_outer_prod_mat = self.up.outer_product();
        // cross product matrix
        let up_cross_mat = self.up.skew_symmetric();
        // rotate
        if self.movement[6] {
            let spin_around_up = Vector3::get_rotate_matrix(up_cross_mat, up_outer_prod_mat, self.rotate_speed);
            self.direction.rotate_vector(spin_around_up);
        }
        if self.movement[7] {
            let spin_around_up = Vector3::get_rotate_matrix(up_cross_mat, up_outer_prod_mat, -self.rotate_speed);
            self.direction.rotate_vector(spin_around_up);
        }

        // spin around left
        // normalise left
        left.normalise();
        // get the outer product
        let left_outer_prod_mat = left.outer_product();
        // cross product matrix
        let left_cross_mat = left.skew_symmetric();
        // rotate
        if self.movement[8] {
            let spin_around_left = Vector3::get_rotate_matrix(left_cross_mat, left_outer_prod_mat, self.rotate_speed);
            self.direction.rotate_vector(spin_around_left);
        }
        if self.movement[9] {
            let spin_around_left = Vector3::get_rotate_matrix(left_cross_mat, left_outer_prod_mat, -self.rotate_speed);
            self.direction.rotate_vector(spin_around_left);
        }
    }
}





impl Vector3 {
    fn get_rotate_matrix(cross_prod_mat: [Vector3; 3], outer_prod_mat: [Vector3; 3], rotate_speed: f32) -> [Vector3; 3] {
        [
            Vector3 {
                x: rotate_speed.cos() + rotate_speed.sin() * cross_prod_mat[0].x + (1.0-rotate_speed.cos()) * outer_prod_mat[0].x, 
                y: rotate_speed.sin() * cross_prod_mat[0].y + (1.0-rotate_speed.cos()) * outer_prod_mat[0].y, 
                z: rotate_speed.sin() * cross_prod_mat[0].z + (1.0-rotate_speed.cos()) * outer_prod_mat[0].z
            },

            Vector3 {
                x: rotate_speed.sin() * cross_prod_mat[1].x + (1.0-rotate_speed.cos()) * outer_prod_mat[1].x, 
                y: rotate_speed.cos() + rotate_speed.sin() * cross_prod_mat[1].y + (1.0-rotate_speed.cos()) * outer_prod_mat[1].y, 
                z: rotate_speed.sin() * cross_prod_mat[1].z + (1.0-rotate_speed.cos()) * outer_prod_mat[1].z
            },

            Vector3 {
                x: rotate_speed.sin() * cross_prod_mat[2].x + (1.0-rotate_speed.cos()) * outer_prod_mat[2].x, 
                y: rotate_speed.sin() * cross_prod_mat[2].y + (1.0-rotate_speed.cos()) * outer_prod_mat[2].y, 
                z: rotate_speed.cos() + rotate_speed.sin() * cross_prod_mat[2].z + (1.0-rotate_speed.cos()) * outer_prod_mat[2].z
            }
        ]
    }

    /// R = Cos(theta)I + sin(theta)Unit_Vector_Cross + (1-cos(theta))Outer_Product
    fn rotate_vector(&mut self, around: [Vector3; 3]) {
        self.x = self.x*around[0].x + self.y*around[0].y + self.z*around[0].z;
        self.y = self.x*around[1].x + self.y*around[1].y + self.z*around[1].z;
        self.z = self.x*around[2].x + self.y*around[2].y + self.z*around[2].z;
    }
}