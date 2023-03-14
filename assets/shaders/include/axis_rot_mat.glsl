// just the code for making a rotation around an unknown axis
mat3 rotationAroundAxis(vec3 axis, float angle) {
    mat3 out_mat;

    out_mat[0].xyz = vec3(
        cos(angle) + axis.x * axis.x * (1.0 - cos(angle)),
        axis.x * axis.y * (1.0 - cos(angle)) - axis.z * sin(angle),
        axis.x * axis.z * (1.0 - cos(angle)) + axis.y * sin(angle)
    );
    out_mat[1].xyz = vec3(
        axis.y * axis.z * (1.0 - cos(angle)) - axis.x * sin(angle),
        cos(angle) + axis.y * axis.y * (1.0 - cos(angle)),
        axis.y * axis.z * (1.0 - cos(angle)) + axis.z * sin(angle)
    );
    out_mat[2].xyz = vec3(
        axis.z * axis.x * (1.0 - cos(angle)) - axis.y * sin(angle),
        axis.z * axis.y * (1.0 - cos(angle)) + axis.x * sin(angle),
        cos(angle) + axis.z * axis.z * (1.0 - cos(angle))
    );

    return out_mat;
}