#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;

uniform mat4 model_matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;

out vec2 fragment_tex_coord;

void main()
{
    fragment_tex_coord = tex_coord;
    gl_Position = projection_matrix * view_matrix * model_matrix  * vec4(position.x, position.y, position.z, 1.0f);
}