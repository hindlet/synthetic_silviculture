#version 450

layout(location = 0) in vec3 position;
layout(location = 2) in vec3 normal;


layout(location = 0) out vec3 v_normal;
layout(location = 2) out vec3 v_position;

layout(set = 0, binding = 0) uniform Data {
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    v_normal = normal;
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);
    v_position = gl_Position.xyz / gl_Position.w;
}