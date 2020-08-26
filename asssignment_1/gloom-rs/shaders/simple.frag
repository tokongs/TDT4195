#version 430 core

in vec2 fragment_tex_coord;
out vec4 color;

uniform sampler2D diffuse_texture;

void main()
{
    color = texture(diffuse_texture, fragment_tex_coord);
}