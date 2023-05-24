#version 450
#include "include/light_maths.glsl"

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;


layout(location = 0) out vec3 v_colour;

layout(set = 0, binding = 0) uniform Data {
    mat4 view;
    mat4 proj;
    vec3 grass_colour;
} uniforms;


layout(set = 0, binding = 1) buffer PointLightData {
    PointLight p_lights[];
};

layout(set = 0, binding = 2) buffer DirLightData {
    DirectionalLight d_lights[];
};


void main() {
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);

    float total_intensity = 0.0;
    for (int i = 0; i < p_lights.length(); i++) {
        total_intensity += get_p_light_intensity(p_lights[i], position);
    }

    for (int i = 0; i < d_lights.length(); i++) {
        total_intensity += get_d_light_intensity(d_lights[i], normal);
    }

    v_colour = uniforms.grass_colour * total_intensity;
}