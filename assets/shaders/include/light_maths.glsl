

struct PointLight {
    vec3 position;
    float intensity;
};

struct DirectionalLight {
    vec3 direction;
    float intensity;
};


float get_p_light_intensity(PointLight light, vec3 target) {
    vec3 dist = target - light.position;
    return light.intensity / abs(dot(dist, dist));
}

float get_d_light_intensity(DirectionalLight light, vec3 target_normal) {
    return max(dot(target_normal, light.direction), 0.0) * light.intensity;
}