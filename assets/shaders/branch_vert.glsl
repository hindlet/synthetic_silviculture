#version 450
#include "include/light_maths.glsl"

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;


layout(location = 0) out vec3 v_colour;


layout(set = 0, binding = 0) uniform Data {
    mat4 view;
    mat4 proj;
    DirectionalLight light;
} uniforms;


const vec3 BROWN = vec3(0.305, 0.208, 0.141);

void main() {
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);

    float total_intensity = get_d_light_intensity(uniforms.light, normal);

    v_colour = total_intensity * BROWN;
}