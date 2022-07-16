#version 330 core

#define M_PI 3.1415926535897932384626433832795

#ifdef BUILDING_VERTEX_SHADER

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_nrm;

uniform mat4 uni_proj;
uniform mat4 uni_view;
uniform mat4 uni_model;

out vec3 var_nrm;

void main() {
    var_nrm = in_nrm;
    gl_Position = uni_proj * uni_view * uni_model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_skybox;

in vec3 var_nrm;

out vec4 out_frag_color;

void main() {
    out_frag_color = vec4(var_nrm, 1.0);
}

#endif
