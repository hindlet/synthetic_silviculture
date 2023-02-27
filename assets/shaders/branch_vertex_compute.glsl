#version 450

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

struct Vertex {
    vec3 position;
    vec3 normal;
};


layout(binding = 0) buffer InVertexBuffer {
    vec3 in_vertices[];
};

layout(binding = 1) buffer OutVertexBuffer {
    Vertex out_vertices[];
    int out_indices[];
};

layout(binding = 2) buffer BranchPolygonVectorBuffer {
    vec3 branch_polygon_vectors[];
};



void main() {

}