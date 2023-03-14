#version 450

// this is just the most simple vertex shader that works for 3d

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);
}