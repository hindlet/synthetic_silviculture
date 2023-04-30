#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;


layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_color;
layout(location = 2) out vec3 v_position;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    v_normal = transpose(inverse(mat3(uniforms.world))) * normal;
    v_color = color;
    gl_Position = uniforms.proj * uniforms.view * uniforms.world * vec4(position, 1.0);
    v_position = gl_Position.xyz / gl_Position.w;
}