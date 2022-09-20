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

noperspective out float frag_dist;
noperspective out vec3 frag_world_pos;
noperspective out vec3 frag_nrm;
noperspective out vec2 frag_uv;
noperspective out vec3 frag_col;

void main() {
    vec4 world_pos = mat_model * vec4(vs_pos, 1.0);
    vec4 eye_pos = mat_view * world_pos;
    vec4 clip_pos = mat_proj * eye_pos;

#ifdef SNAP_VERTEX_POS
    clip_pos = snap_pos(clip_pos, render_res);
#endif

    frag_world_pos = world_pos.xyz;
    frag_nrm = normalize(mat_normal * vs_normal);
    frag_uv = vs_uv;
    frag_dist = length(eye_pos);
    gl_Position = clip_pos;
    frag_col = vs_col.rgb;
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_base_color;

noperspective in float frag_dist;
noperspective in vec3 frag_world_pos;
noperspective in vec3 frag_nrm;
noperspective in vec2 frag_uv;
noperspective in vec3 frag_col;

out vec4 out_frag_color;

void main() {
    // Sample base color texture and calculate base color and alpha
    vec4 base_color_tex = has_base_color_texture ? texture(tex_base_color, frag_uv) : vec4(1.0);
    vec3 albedo = base_color.rgb * base_color_tex.rgb;
    float alpha = base_color.a * base_color_tex.a;

    // Alpha clip low opacity fragments
    if (alpha < 0.1)
        discard;

    // For the catstation logo, we want to add some basic shading on top with a directional light.
    // I found it was easier to write a shader for it than to get blender to produce the exact
    // result I wanted...
    vec3 forward = normalize(vec3(-1.0, 0.1, 1.0));
    float diffuse = dot(forward, frag_nrm);

    // Calculate vertex lighting for fragment
    const float AMBIENT_LIGHT = 0.3;
    vec3 light = frag_col * min(1.0, lighting_strength * diffuse + AMBIENT_LIGHT);

    // Calculate foggedness of fragment
    float fog_factor = fog_dist.y > 0.0 && fog_dist.y > fog_dist.x ?
        clamp((frag_dist - fog_dist.x)/(fog_dist.y - fog_dist.x), 0.0, 1.0)
        : 0.0;

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
