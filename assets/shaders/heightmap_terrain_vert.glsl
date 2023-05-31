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
    vec3 rock_colour;
    float grass_slope_threshold;
    float grass_blend_amount;
} uniforms;



void main() {
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);

    float slope = 1 - normal.y;
    float grass_blend_height = uniforms.grass_slope_threshold * (1 - uniforms.grass_blend_amount);
    float grass_weight = 1 - clamp((slope - grass_blend_height) / (uniforms.grass_slope_threshold - grass_blend_height), 0.0, 1.0);

    float total_intensity = get_d_light_intensity(uniforms.light, normal);

    v_colour = total_intensity * (grass_weight * uniforms.grass_colour + (1-grass_weight) * uniforms.rock_colour);
}