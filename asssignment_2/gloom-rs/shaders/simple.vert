#version 430 core

in vec3 position;
in vec4 color;

out vec4 vertexColor;

uniform mat4 projection;
uniform mat4 view;

void main()
{
    vertexColor = color;
    gl_Position = projection * view * vec4(position, 1.0f);
}