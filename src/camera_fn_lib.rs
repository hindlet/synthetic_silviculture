


pub struct Camera {
    pub position: [f32; 3], 
    pub direction: [f32; 3],
    pub up: [f32; 3],   
    pub movement: [bool; 10], // forward, back, left, left, up, down, spin right, spin left, spin forward, spin backward
}


// gets the camera view matrix
pub fn camera_view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}





// made me suffer but it does work haha i wanna die now
#[allow(unused_parens)]
pub fn camera_move(camera: &mut Camera) {
    let speed = 0.5 * 0.0166667;
    let rotate_speed: f32 = 0.02;

    // take cross of direction and up to get left
    let left = {
        let a = camera.direction;
        let b = camera.up;
        [ a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0]]
    };

    let forward = {
        let a = left;
        let b = camera.up;
        [ a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0]]
    };

    // forward/back
    if camera.movement[0] {camera.position = [camera.position[0]-forward[0]*speed, camera.position[1]-forward[1]*speed, camera.position[2]-forward[2]*speed]}
    if camera.movement[1] {camera.position = [camera.position[0]+forward[0]*speed, camera.position[1]+forward[1]*speed, camera.position[2]+forward[2]*speed]}
    // left/right
    if camera.movement[2] {camera.position = [camera.position[0]+left[0]*speed, camera.position[1]+left[1]*speed, camera.position[2]+left[2]*speed]}
    if camera.movement[3] {camera.position = [camera.position[0]-left[0]*speed, camera.position[1]-left[1]*speed, camera.position[2]-left[2]*speed]}
    // up/down
    if camera.movement[4] {camera.position = [camera.position[0], camera.position[1] + speed, camera.position[2]]}
    if camera.movement[5] {camera.position = [camera.position[0], camera.position[1] - speed, camera.position[2]]}

    // spin around y axis
    if camera.movement[6] {
        let spin_around_y_axis = [
        [rotate_speed.cos(), 0.0 , rotate_speed.sin()],
        [0.0, 1.0, 0.0],
        [-rotate_speed.sin(), 0.0, rotate_speed.cos()],];
        camera.direction = [
        camera.direction[0]*spin_around_y_axis[0][0] + camera.direction[1]*spin_around_y_axis[0][1] + camera.direction[2]*spin_around_y_axis[0][2],
        camera.direction[0]*spin_around_y_axis[1][0] + camera.direction[1]*spin_around_y_axis[1][1] + camera.direction[2]*spin_around_y_axis[1][2],
        camera.direction[0]*spin_around_y_axis[2][0] + camera.direction[1]*spin_around_y_axis[2][1] + camera.direction[2]*spin_around_y_axis[2][2],]}
    if camera.movement[7] {
        let spin_around_y_axis = [
        [(-rotate_speed).cos(), 0.0 , (-rotate_speed).sin()],
        [0.0, 1.0, 0.0],
        [-(-rotate_speed).sin(), 0.0, (-rotate_speed).cos()],];
        camera.direction = [
        camera.direction[0]*spin_around_y_axis[0][0] + camera.direction[1]*spin_around_y_axis[0][1] + camera.direction[2]*spin_around_y_axis[0][2],
        camera.direction[0]*spin_around_y_axis[1][0] + camera.direction[1]*spin_around_y_axis[1][1] + camera.direction[2]*spin_around_y_axis[1][2],
        camera.direction[0]*spin_around_y_axis[2][0] + camera.direction[1]*spin_around_y_axis[2][1] + camera.direction[2]*spin_around_y_axis[2][2],]}

    // spin around left
    // first we must get a unit vector for left
    let left_unit = {
        let left_length = (left[0]*left[0] + left[1]*left[1] + left[2]*left[2]).sqrt();
        [left[0] / left_length, left[1] / left_length, left[2] / left_length]
    };
    // then we get the outer product for that unit vector
    let left_unit_outer_product = [
        [left_unit[0] * left_unit[0], left_unit[0] * left_unit[1], left_unit[0] * left_unit[2]],
        [left_unit[1] * left_unit[0], left_unit[1] * left_unit[1], left_unit[1] * left_unit[2]],
        [left_unit[2] * left_unit[0], left_unit[2] * left_unit[1], left_unit[2] * left_unit[2]]];
    // and finally the cross product matrix of the unit vector
    let left_unit_cross = [
        [0.0, -left_unit[2], left_unit[1]],
        [left_unit[2], 0.0, -left_unit[0]],
        [-left_unit[1], left_unit[1], 0.0],
    ];
    // finally in these we use R = Cos(theta)I + sin(theta)Unit_Vector_Cross + (1-cos(theta))Outer_Product
    if camera.movement[8] {
        let spin_around_left = [
            [rotate_speed.cos() + rotate_speed.sin()*left_unit_cross[0][0] + (1.0-rotate_speed.cos())*left_unit_outer_product[0][0], 
            rotate_speed.sin()*left_unit_cross[0][1] + (1.0-rotate_speed.cos())*left_unit_outer_product[0][1], 
            rotate_speed.sin()*left_unit_cross[0][2] + (1.0-rotate_speed.cos())*left_unit_outer_product[0][2]],

            [rotate_speed.sin()*left_unit_cross[1][0] + (1.0-rotate_speed.cos())*left_unit_outer_product[1][0], 
            rotate_speed.cos() + rotate_speed.sin()*left_unit_cross[1][1] + (1.0-rotate_speed.cos())*left_unit_outer_product[1][1], 
            rotate_speed.sin()*left_unit_cross[1][2] + (1.0-rotate_speed.cos())*left_unit_outer_product[1][2]],

            [rotate_speed.sin()*left_unit_cross[2][0] + (1.0-rotate_speed.cos())*left_unit_outer_product[2][0], 
            rotate_speed.sin()*left_unit_cross[2][1] + (1.0-rotate_speed.cos())*left_unit_outer_product[2][1], 
            rotate_speed.cos() + rotate_speed.sin()*left_unit_cross[2][2] + (1.0-rotate_speed.cos())*left_unit_outer_product[2][2]],
        ];
        camera.direction = [
        camera.direction[0]*spin_around_left[0][0] + camera.direction[1]*spin_around_left[0][1] + camera.direction[2]*spin_around_left[0][2],
        camera.direction[0]*spin_around_left[1][0] + camera.direction[1]*spin_around_left[1][1] + camera.direction[2]*spin_around_left[1][2],
        camera.direction[0]*spin_around_left[2][0] + camera.direction[1]*spin_around_left[2][1] + camera.direction[2]*spin_around_left[2][2],]
    }

    if camera.movement[9] {
        let spin_speeeeeeeed = -rotate_speed;
        let spin_around_left = [
            [spin_speeeeeeeed.cos() + spin_speeeeeeeed.sin()*left_unit_cross[0][0] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[0][0], 
            spin_speeeeeeeed.sin()*left_unit_cross[0][1] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[0][1], 
            spin_speeeeeeeed.sin()*left_unit_cross[0][2] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[0][2]],

            [spin_speeeeeeeed.sin()*left_unit_cross[1][0] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[1][0], 
            spin_speeeeeeeed.cos() + spin_speeeeeeeed.sin()*left_unit_cross[1][1] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[1][1], 
            spin_speeeeeeeed.sin()*left_unit_cross[1][2] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[1][2]],

            [spin_speeeeeeeed.sin()*left_unit_cross[2][0] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[2][0], 
            spin_speeeeeeeed.sin()*left_unit_cross[2][1] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[2][1], 
            spin_speeeeeeeed.cos() + spin_speeeeeeeed.sin()*left_unit_cross[2][2] + (1.0-spin_speeeeeeeed.cos())*left_unit_outer_product[2][2]],
        ];
        camera.direction = [
        camera.direction[0]*spin_around_left[0][0] + camera.direction[1]*spin_around_left[0][1] + camera.direction[2]*spin_around_left[0][2],
        camera.direction[0]*spin_around_left[1][0] + camera.direction[1]*spin_around_left[1][1] + camera.direction[2]*spin_around_left[1][2],
        camera.direction[0]*spin_around_left[2][0] + camera.direction[1]*spin_around_left[2][1] + camera.direction[2]*spin_around_left[2][2],]
    }
}