#version 430 core

in vec3 in_position;
in vec3 in_normal;
in vec4 in_color;

out vec4 color;
out vec3 normal;

uniform mat4 vp_matrix;
uniform mat4 model_matrix;

void main()
{
    color = in_color;
    normal = normalize(mat3(model_matrix) * in_normal);
    gl_Position = vp_matrix * model_matrix * vec4(in_position, 1.0f);
}