#version 330 core

#define M_PI 3.1415926535897932384626433832795

#ifdef BUILDING_VERTEX_SHADER

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec2 in_uv;

uniform mat4 uni_proj;
uniform mat4 uni_view;
uniform mat4 uni_model;

out vec2 var_uv;

void main() {
    var_uv = in_uv;
    gl_Position = uni_proj * uni_view * uni_model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_skybox;

in vec2 var_uv;

out vec4 out_frag_color;

void main() {
    vec3 offset = vec3(var_uv-0.5,0)*2;
    vec3 d = vec3(0.0, 0.0, -1.0);
    vec2 tx = vec2(0.5 + atan(d.z, sqrt(d.x*d.x + d.y*d.y))/(2.0 * M_PI), 0.5 + atan(d.y, d.x)/(2.0 * M_PI));

    vec4 c = texture2D( tex_skybox, tx);

    out_frag_color = texture(tex_skybox, var_uv);
    out_frag_color = vec4(1.0, 0.0, 1.0, 1.0);
}

#endif
