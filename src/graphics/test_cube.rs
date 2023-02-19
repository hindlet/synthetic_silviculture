use bytemuck::{Pod, Zeroable};
use crate::graphics::general_graphics::{Vertex, Normal};


pub const VERTICES: [Vertex; 8] = [
    Vertex {position: [-0.5, -0.5, -0.5]},
    Vertex {position: [0.5, -0.5, -0.5]},
    Vertex {position: [-0.5, -0.5, 0.5]},
    Vertex {position: [0.5, -0.5, 0.5]},
    Vertex {position: [-0.5, 0.5, -0.5]},
    Vertex {position: [0.5, 0.5, -0.5]},
    Vertex {position: [-0.5, 0.5, 0.5]},
    Vertex {position: [0.5, 0.5, 0.5]}
];

pub const NORMALS: [Normal; 8] = [
    Normal {normal: [-1.0, -1.0, -1.0]},
    Normal {normal: [1.0, -1.0, -1.0]},
    Normal {normal: [-1.0, -1.0, 1.0]},
    Normal {normal: [1.0, -1.0, 1.0]},
    Normal {normal: [-1.0, 1.0, -1.0]},
    Normal {normal: [1.0, 1.0, -1.0]},
    Normal {normal: [-1.0, 1.0, 1.0]},
    Normal {normal: [1.0, 1.0, 1.0]},
];

pub const INDICES: [u16; 36] = [
    0, 4, 1,
    4, 1, 5,
    1, 5, 3,
    5, 3, 7,
    3, 7, 2,
    7, 2, 6,
    2, 6, 0,
    6, 0, 4,
    0, 2, 1,
    2, 1, 3,
    4, 6, 5,
    6, 5, 7
];