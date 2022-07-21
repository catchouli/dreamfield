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

vec3 calc_ray_dir() {
    vec2 pos = var_uv * 2.0 - 1.0;

    vec4 v = mat_view_proj_inv * vec4(pos, -1.0, 1.0);
    vec3 start = v.xyz / v.w;

    v = mat_view_proj_inv * vec4(pos, 0.0, 1.0);
    vec3 end = v.xyz / v.w;

    return normalize(end - start);
}

vec2 project_equilateral(vec3 ray_dir) {
    float longitude = ray_dir.x;
    float latitude = ray_dir.y;

    float x = longitude / M_PI;
    float y = log((1 + sin(latitude))/(1 - sin(latitude))) / (4.0 * M_PI);

    return vec2(x, y);
}

void main() {
    float horz_fov = M_PI / 2.0 * vp_aspect;
    float vert_fov = M_PI / 2.0;

    // Calculate ray dir
    vec3 ray_dir = calc_ray_dir();

    out_frag_color = 0.25 * texture(tex_skybox, project_equilateral(ray_dir));
}

#endif
