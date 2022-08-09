#version 330 core

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

uniform sampler2D tex;

in vec2 var_uv;

out vec4 out_frag_color;

void main() {
    // https://uk.mathworks.com/help/releases/R2020a/images/ref/ntsc2rgb.html#mw_0a7b75f5-1fde-400a-ad3c-68208bdaf07e
    const mat3 yiq_to_rgb = mat3(1.0, 1.0, 1.0, 0.956, -0.272, -1.106, 0.621, -0.647, 1.703);

    float y = textureLod(tex, var_uv, 0.0).r;
    float i = textureLod(tex, var_uv, 2.0).g;
    float q = textureLod(tex, var_uv, 4.0).b;

    // Convert to rgb
    vec3 rgb = yiq_to_rgb * vec3(y, i, q);

    out_frag_color = vec4(rgb, 1.0);
}

#endif
