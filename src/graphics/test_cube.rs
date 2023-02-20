use bytemuck::{Pod, Zeroable};
use crate::graphics::general_graphics::{Vertex, Normal};


pub const VERTICES: [Vertex; 8] = [
    Vertex {position: [-0.5, -0.5, -0.5], color: [0.84, 0.01, 0.44]},
    Vertex {position: [0.5, -0.5, -0.5], color: [0.61, 0.31, 0.59]},
    Vertex {position: [-0.5, -0.5, 0.5], color: [0.61, 0.31, 0.59]},
    Vertex {position: [0.5, -0.5, 0.5], color: [0.0, 0.22, 0.66]},
    Vertex {position: [-0.5, 0.5, -0.5], color: [0.84, 0.01, 0.44]},
    Vertex {position: [0.5, 0.5, -0.5], color: [0.61, 0.31, 0.59]},
    Vertex {position: [-0.5, 0.5, 0.5], color: [0.61, 0.31, 0.59]},
    Vertex {position: [0.5, 0.5, 0.5], color: [0.0, 0.22, 0.66]}
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