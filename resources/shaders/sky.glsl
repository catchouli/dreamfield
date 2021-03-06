#version 330 core

#include resources/shaders/include/constants.glsl
#include resources/shaders/include/uniforms.glsl

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
    float horz_fov = M_PI / 2.0 * target_aspect;
    float vert_fov = M_PI / 2.0;

    vec3 ray_dir = calc_ray_dir();
    vec3 rd = ray_dir;
    //vec2 tex_coords = project_equilateral(ray_dir);

    vec2 tex_coords = vec2(atan(rd.z, rd.x) + M_PI, acos(rd.y)) / vec2(2.0 * M_PI, M_PI);

    vec3 out_color = texture(tex_skybox, tex_coords).rgb;

    out_frag_color = vec4(out_color, 1.0);
}

#endif
