#version 400 core

#define SNAP_VERTEX_POS

#include resources/shaders/include/uniforms.glsl
#include resources/shaders/include/utils.glsl

#ifdef BUILDING_VERTEX_SHADER

layout(location = 0) in vec3 vs_pos;
layout(location = 1) in vec3 vs_normal;
layout(location = 3) in vec2 vs_uv;
layout(location = 5) in vec4 vs_col;
layout(location = 6) in vec4 vs_joint;
layout(location = 7) in vec4 vs_weight;

#ifdef TESSELLATION_ENABLED
out vec4 tcs_clip_pos;
out vec3 tcs_eye_pos;
out vec3 tcs_world_pos;
out vec3 tcs_normal;
out vec2 tcs_uv;
out vec3 tcs_col;
#else
out float frag_dist;
out vec3 frag_world_pos;
out vec3 frag_nrm;
out vec2 frag_uv;
out vec3 frag_light;
#endif

void main() {
    mat4 skin_matrix =
        vs_weight.x * joints[int(vs_joint.x)].joint_matrix +
        vs_weight.y * joints[int(vs_joint.y)].joint_matrix +
        vs_weight.z * joints[int(vs_joint.z)].joint_matrix +
        vs_weight.w * joints[int(vs_joint.w)].joint_matrix;

    vec4 world_pos = skinning_enabled
        ? skin_matrix * vec4(vs_pos, 1.0)
        : mat_model * vec4(vs_pos, 1.0);

    vec4 eye_pos = mat_view * world_pos;
    vec4 clip_pos = mat_proj * eye_pos;

#ifdef SNAP_VERTEX_POS
    clip_pos = snap_pos(clip_pos, render_res);
#endif

#ifdef TESSELLATION_ENABLED
    tcs_clip_pos = clip_pos;
    tcs_eye_pos = eye_pos.xyz;
    tcs_world_pos = world_pos.xyz;

    tcs_normal = vs_normal;
    tcs_uv = vs_uv;
    tcs_col = vs_col.rgb;
#else
    frag_world_pos = world_pos.xyz;
    frag_nrm = normalize(mat_normal * vs_normal);
    frag_uv = vs_uv;
    frag_dist = length(eye_pos);
    gl_Position = clip_pos;
    frag_light = vs_col.rgb;
#endif
}

#endif

#ifdef BUILDING_TESS_CONTROL_SHADER

in vec4 tcs_clip_pos[];
in vec3 tcs_eye_pos[];
in vec3 tcs_world_pos[];
in vec3 tcs_normal[];
in vec2 tcs_uv[];
in vec3 tcs_col[];

layout(vertices=3) out;
out vec4 tes_clip_pos[];
out vec3 tes_eye_pos[];
out vec3 tes_world_pos[];
out vec3 tes_normal[];
out vec2 tes_uv[];
out vec3 tes_col[];

void main() {
    tes_clip_pos[gl_InvocationID] = tcs_clip_pos[gl_InvocationID];
    tes_eye_pos[gl_InvocationID] = tcs_eye_pos[gl_InvocationID];
    tes_world_pos[gl_InvocationID] = tcs_world_pos[gl_InvocationID];
    tes_normal[gl_InvocationID] = tcs_normal[gl_InvocationID];
    tes_uv[gl_InvocationID] = tcs_uv[gl_InvocationID];
    tes_col[gl_InvocationID] = tcs_col[gl_InvocationID];

    float eye_dist = length(tes_eye_pos[gl_InvocationID]);

    float min_tess_level = 1.0;
    float max_tess_level = 6.0;
    float tess_end = 10.0;

    float tess_level_range = max_tess_level - min_tess_level;
    float dist_norm = min(eye_dist, tess_end) / tess_end;
    float tess_level = min_tess_level + (1.0 - dist_norm) * tess_level_range;

    gl_TessLevelInner[0] = tess_level;
    gl_TessLevelOuter[0] = tess_level;
    gl_TessLevelOuter[1] = tess_level;
    gl_TessLevelOuter[2] = tess_level;
}

#endif

#ifdef BUILDING_TESS_EVAL_SHADER

layout(triangles,equal_spacing) in;
in vec4 tes_clip_pos[];
in vec3 tes_eye_pos[];
in vec3 tes_world_pos[];
in vec3 tes_normal[];
in vec2 tes_uv[];
in vec3 tes_col[];

out float frag_dist;
out vec3 frag_world_pos;
out vec3 frag_nrm;
out vec2 frag_uv;

out vec3 frag_light;

vec4 lerp3D(vec4 v0, vec4 v1, vec4 v2)
{
    return vec4(gl_TessCoord.x) * v0 + vec4(gl_TessCoord.y) * v1 + vec4(gl_TessCoord.z) * v2;
}

vec3 lerp3D(vec3 v0, vec3 v1, vec3 v2)
{
    return vec3(gl_TessCoord.x) * v0 + vec3(gl_TessCoord.y) * v1 + vec3(gl_TessCoord.z) * v2;
}

void main() {
    vec4 clip_pos = lerp3D(tes_clip_pos[0], tes_clip_pos[1], tes_clip_pos[2]);
    vec3 eye_pos = lerp3D(tes_eye_pos[0], tes_eye_pos[1], tes_eye_pos[2]);
    vec3 world_pos = lerp3D(tes_world_pos[0], tes_world_pos[1], tes_world_pos[2]);
    vec3 normal = lerp3D(tes_normal[0], tes_normal[1], tes_normal[2]);
    vec2 uv = lerp3D(vec3(tes_uv[0], 0.0), vec3(tes_uv[1], 0.0), vec3(tes_uv[2], 0.0)).xy;
    vec3 col = lerp3D(tes_col[0], tes_col[1], tes_col[2]);

    frag_world_pos = world_pos;
    frag_nrm = normalize(mat_normal * normal);
    frag_uv = uv;
    frag_dist = length(eye_pos);
    gl_Position = clip_pos;
    frag_light = col;
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_base_color;

in float frag_dist;
in vec3 frag_world_pos;
in vec3 frag_nrm;
in vec2 frag_uv;
in vec3 frag_light;

out vec4 out_frag_color;

void main() {
    // Sample base color texture and calculate base color and alpha
    vec4 base_color_tex = has_base_color_texture ? texture(tex_base_color, frag_uv) : vec4(1.0);
    vec3 albedo = base_color.rgb * base_color_tex.rgb;
    float alpha = base_color.a * base_color_tex.a;

    // Alpha clip low opacity fragments
    if (alpha < 0.1)
        discard;

    // Calculate vertex lighting for fragment
    const vec3 AMBIENT_LIGHT = vec3(0.01);
    vec3 light = 1.5 * frag_light * lighting_strength + AMBIENT_LIGHT;

    // Calculate foggedness of fragment
    float fog_factor = fog_dist.y > 0.0 && fog_dist.y > fog_dist.x ?
        clamp((frag_dist - fog_dist.x)/(fog_dist.y - fog_dist.x), 0.0, 1.0)
        : 1.0;

    // Multiply the light by the albedo and get the fragment's color and value
    vec3 pre_dither_color = light * albedo;
    float pre_dither_value = luma(pre_dither_color);

    // Calculate the dithering strength using the value, skewing it exponentially towards 1
    const float DITHER_EXPONENT = 0.5;
    float dither_strength = pow(pre_dither_value, DITHER_EXPONENT);

    // Apply dithering to fragment
    vec3 post_dither_color = dither(pre_dither_color, ivec2(gl_FragCoord.xy), dither_strength);

    // Apply fog to fragment
    vec3 post_fog_color = post_dither_color * (1.0 - fog_factor)
        + fog_factor * fog_color;

    out_frag_color = vec4(post_fog_color, alpha);
}

#endif
