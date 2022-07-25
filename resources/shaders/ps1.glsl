#version 400 core

#define VERTEX_LIGHTING 1

#include resources/shaders/include/uniforms.glsl
#include resources/shaders/include/lighting.glsl
#include resources/shaders/include/utils.glsl

#ifdef BUILDING_VERTEX_SHADER

layout(location = 0) in vec3 vs_pos;
layout(location = 1) in vec3 vs_normal;
layout(location = 3) in vec2 vs_uv;

out vec3 tcs_pos;
out vec3 tcs_normal;
out vec2 tcs_uv;

void main() {
    tcs_pos = vs_pos;
    tcs_normal = vs_normal;
    tcs_uv = vs_uv;
}

#endif

#ifdef BUILDING_TESS_CONTROL_SHADER

in vec3 tcs_pos[];
in vec3 tcs_normal[];
in vec2 tcs_uv[];

layout(vertices=3) out;
out vec3 tes_pos[];
out vec3 tes_normal[];
out vec2 tes_uv[];

void main() {
    tes_pos[gl_InvocationID] = tcs_pos[gl_InvocationID];
    tes_normal[gl_InvocationID] = tcs_normal[gl_InvocationID];
    tes_uv[gl_InvocationID] = tcs_uv[gl_InvocationID];

    vec4 eye_pos = mat_view * mat_model * vec4(tcs_pos[gl_InvocationID], 1.0);
    float eye_dist = length(eye_pos);

    float min_tess_level = 1.0;
    float max_tess_level = 4.0;
    float tess_end = 15.0;
    float dist_norm = min(eye_dist, tess_end) / tess_end;
    float tess_level = min_tess_level + (1.0 - dist_norm) * (max_tess_level - min_tess_level);

    gl_TessLevelInner[0] = tess_level;
    gl_TessLevelOuter[0] = tess_level;
    gl_TessLevelOuter[1] = tess_level;
    gl_TessLevelOuter[2] = tess_level;
}

#endif

#ifdef BUILDING_TESS_EVAL_SHADER

layout(triangles,equal_spacing) in;
in vec3 tes_pos[];
in vec3 tes_normal[];
in vec2 tes_uv[];

noperspective out float frag_dist;
noperspective out vec3 frag_world_pos;
noperspective out vec3 frag_nrm;
noperspective out vec2 frag_uv;

#ifdef VERTEX_LIGHTING
noperspective out vec3 frag_light;
#endif

vec3 lerp3D(vec3 v0, vec3 v1, vec3 v2)
{
    return vec3(gl_TessCoord.x) * v0 + vec3(gl_TessCoord.y) * v1 + vec3(gl_TessCoord.z) * v2;
}

void main() {
    vec3 pos = lerp3D(tes_pos[0], tes_pos[1], tes_pos[2]);
    vec3 normal = lerp3D(tes_normal[0], tes_normal[1], tes_normal[2]);
    vec2 uv = lerp3D(vec3(tes_uv[0], 0.0), vec3(tes_uv[1], 0.0), vec3(tes_uv[2], 0.0)).xy;

    vec4 world_pos = mat_model * vec4(pos, 1.0);
    vec4 eye_pos = mat_view * world_pos;
    vec4 clip_pos = mat_proj * eye_pos;

    frag_world_pos = world_pos.xyz;
    frag_nrm = normalize(mat_normal * normal);
    frag_uv = uv;
    frag_dist = length(eye_pos);
    gl_Position = snap_pos(clip_pos, render_res);

#ifdef VERTEX_LIGHTING
    frag_light = calculate_lighting(frag_world_pos, frag_nrm);
#endif
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_base_color;

noperspective in float frag_dist;
noperspective in vec3 frag_world_pos;
noperspective in vec3 frag_nrm;
noperspective in vec2 frag_uv;

#ifdef VERTEX_LIGHTING
noperspective in vec3 frag_light;
#endif

out vec4 out_frag_color;

void main() {
    vec3 sun_dir = normalize(vec3(0.5, 0.5, 0.5));
    vec3 base_color = has_base_color_texture ? texture(tex_base_color, frag_uv).xyz : vec3(1.0);
    float diffuse_strength = dot(sun_dir, frag_nrm);

#ifdef VERTEX_LIGHTING
    vec3 light = frag_light;
#else
    vec3 light = calculate_lighting(frag_world_pos, frag_nrm);
#endif

    float fog_factor = fog_dist.y > 0.0 && fog_dist.y > fog_dist.x ?
        clamp((frag_dist - fog_dist.x)/(fog_dist.y - fog_dist.x), 0.0, 1.0)
        : 1.0;

    vec3 out_color = light * base_color * (1.0 - fog_factor)
        + fog_factor * fog_color;

    out_color = min(out_color, vec3(1.0));
    out_frag_color = vec4(out_color, 1.0);
}

#endif
