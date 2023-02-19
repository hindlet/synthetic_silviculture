#version 450

layout(location = 0) in vec3 v_normal;

layout(location = 0) out vec4 f_colour;

const vec3 LIGHT = vec3(0.0, 0.5, 1.0);

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark = vec3(0.6, 0.0, 0.0);
    vec3 regular = vec3(1.0, 0.0, 0.0);

    f_colour = vec4(mix(dark, regular, brightness), 1.0);
}