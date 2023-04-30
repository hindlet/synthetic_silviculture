#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 v_color;
layout(location = 2) in vec3 v_position;

layout(location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(0.0, 0.5, 1.0);
const float AMBIENT = 0.3;
const float DIFFUSE = 0.8;


void main() {
    vec3 ambient_color = AMBIENT * v_color;
    vec3 diffuse_color = DIFFUSE * v_color;

    float diffuse = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);


    f_color = vec4(ambient_color + diffuse * diffuse_color, 1.0);
}