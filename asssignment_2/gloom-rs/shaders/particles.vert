#version 430 core
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 particle_position;
layout(location = 2) in vec4 particle_color;

out vec4 vertexColor;

uniform mat4 projection;
uniform mat4 view;

void main()
{
    vec3 cam_right = vec3(view[0][0], view[1][0], view[2][0]);
    vec3 cam_up = vec3(view[0][1], view[1][1], view[2][1]);

    vertexColor = particle_color;
    gl_Position = projection * view * vec4(((position.x * cam_right + position.y * cam_up))* 0.005 + particle_position, 1.0f);
}