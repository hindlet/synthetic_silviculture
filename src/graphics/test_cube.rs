use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32)
}

implement_vertex!(Vertex, position);

pub const VERTICES: [Vertex; 8] = [
    Vertex {position: (0.0, 0.0, 0.0)},
    Vertex {position: (1.0, 0.0, 0.0)},
    Vertex {position: (0.0, 1.0, 0.0)},
    Vertex {position: (1.0, 1.0, 0.0)},
    Vertex {position: (0.0, 0.0, 1.0)},
    Vertex {position: (1.0, 0.0, 1.0)},
    Vertex {position: (0.0, 1.0, 1.0)},
    Vertex {position: (1.0, 1.0, 1.0)},
];

#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32)
}

implement_vertex!(Normal, normal);

pub const NORMALS: [Normal; 8] = [
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
    Normal {normal: (0.0, 1.0, 0.0)},
];

pub const INDICES: [u16; 36] = [
    0, 1, 3,
    3, 0, 2,

    1, 7, 5,
    7, 1, 3,

    4, 2, 0,
    2, 4, 6,

    5, 6, 4,
    6, 5, 7,

    2, 7, 3,
    7, 2, 6,

    0, 5, 1,
    5, 0, 4
];