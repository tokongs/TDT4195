#version 430 core

out vec4 out_color;

in vec3 normal;
in vec4 color;

vec3 lightDirection = normalize(vec3(0.8, -0.5,0.6));

void main()
{
    vec4 lightColor = color * max(0, dot(normal, -lightDirection));
    out_color = vec4(lightColor.xyz, color.a);
}