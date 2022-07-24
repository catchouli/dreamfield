#version 330 core

#define VERTEX_LIGHTING

#include resources/shaders/include/uniforms.glsl
#include resources/shaders/include/lighting.glsl
#include resources/shaders/include/utils.glsl

#ifdef BUILDING_VERTEX_SHADER

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_nrm;
layout (location = 3) in vec2 in_uv;

noperspective out float var_dist;
noperspective out vec3 var_world_pos;
noperspective out vec3 var_nrm;
noperspective out vec2 var_uv;

#ifdef VERTEX_LIGHTING
noperspective out vec3 var_light;
#endif

void main() {
    vec4 world_pos = mat_model * vec4(in_pos, 1.0);
    vec4 eye_pos = mat_view * world_pos;
    vec4 clip_pos = mat_proj * eye_pos;

    var_world_pos = world_pos.xyz;
    var_nrm = normalize(mat_normal * in_nrm);
    var_uv = in_uv;
    var_dist = length(eye_pos);
    gl_Position = snap_pos(clip_pos, render_res);

#ifdef VERTEX_LIGHTING
    var_light = calculate_lighting(var_world_pos, var_nrm);
#endif
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_base_color;

noperspective in float var_dist;
noperspective in vec3 var_world_pos;
noperspective in vec3 var_nrm;
noperspective in vec2 var_uv;

#ifdef VERTEX_LIGHTING
noperspective in vec3 var_light;
#endif

out vec4 out_frag_color;

void main() {
    vec3 sun_dir = normalize(vec3(0.5, 0.5, 0.5));
    vec3 base_color = has_base_color_texture ? texture(tex_base_color, var_uv).xyz : vec3(1.0);
    float diffuse_strength = dot(sun_dir, var_nrm);

#ifdef VERTEX_LIGHTING
    vec3 light = var_light;
#else
    vec3 light = calculate_lighting(var_world_pos, var_nrm);
#endif

    float fog_factor = fog_dist.y > 0.0 && fog_dist.y > fog_dist.x ?
        clamp((var_dist - fog_dist.x)/(fog_dist.y - fog_dist.x), 0.0, 1.0)
        : 1.0;

    vec3 out_color = light * base_color * (1.0 - fog_factor)
        + fog_factor * fog_color;

    out_color = min(out_color, vec3(1.0));
    out_frag_color = vec4(out_color, 1.0);
}

#endif
