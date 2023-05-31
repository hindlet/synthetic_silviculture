#version 450
#include "include/light_maths.glsl"

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;


layout(location = 0) out vec3 v_colour;

layout(set = 0, binding = 0) uniform Data {
    mat4 view;
    mat4 proj;
    DirectionalLight light;
    vec3 grass_colour;
} uniforms;



void main() {
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);

    float total_intensity = get_d_light_intensity(uniforms.light, normal);

    v_colour = uniforms.grass_colour * total_intensity;
}