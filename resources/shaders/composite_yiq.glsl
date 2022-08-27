#version 330 core

#include resources/shaders/include/uniforms.glsl
#include resources/shaders/include/utils.glsl

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
    const mat3 rgb_to_yiq = mat3(0.299, 0.596, 0.211, 0.587, -0.274, -0.523, 0.114, -0.322, 0.312);

    // Sample texture
    vec3 rgb = linear_to_srgb(texture(tex, var_uv).rgb);

    // Downsample to 5-bit (32 colors)
    rgb = floor(rgb * 32.0) / 32.0;

    // Convert to yiq
    vec3 yiq = rgb_to_yiq * rgb;

    out_frag_color = vec4(yiq, 1.0);
}

#endif
