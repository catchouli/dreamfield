#version 330 core

#include resources/shaders/include/uniforms.glsl
#define M_PI 3.1415926535897932384626433832795

#ifdef BUILDING_VERTEX_SHADER

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec2 in_uv;

out vec2 var_uv;

void main() {
    var_uv = in_uv;
    gl_Position = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_skybox;

in vec2 var_uv;

out vec4 out_frag_color;

void main() {
    const float horz_fov = M_PI / 2.0;
    const float vert_fov = M_PI / 2.0;

    // Scale uv from -1 to 1 and put 0 in center, and then use it to calculate a ray direction
    vec2 uv = vec2(1.0 - var_uv.x, var_uv.y) * 2.0 - 1.0;
    vec3 rd = mat3(mat_view) * normalize(vec3(uv.xy * vec2(tan(0.5 * horz_fov), tan(0.5 * vert_fov)), 1.0));

    // Equirectangular projection
    vec2 tex_coord = vec2(atan(rd.z, rd.x) + M_PI, acos(-rd.y)) / vec2(2.0 * M_PI, M_PI);

    out_frag_color = texture(tex_skybox, tex_coord);
}

#endif
