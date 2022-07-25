#version 330 core

#include dreamfield_renderer/shaders/include/uniforms.glsl

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

uniform sampler2D blit_tex;

in vec2 var_uv;

out vec4 out_frag_color;

// Returns whether the specified sample is in the range (0..1, 0..1)
bool sample_in_texture(vec2 sample) {
    return sample.x >= 0.0 && sample.y >= 0.0 && sample.x <= 1.0 && sample.y <= 1.0;
}

void main() {
    // Calculate difference between window and target aspect ratio
    float aspect_scale = target_aspect / window_aspect;

    // Scale width or height depending on whether the window aspect ratio is bigger or smaller than expected
    vec2 sample_uv = window_aspect > target_aspect
        ? vec2(var_uv.x / aspect_scale + 0.5 - 0.5 / aspect_scale, var_uv.y)
        : vec2(var_uv.x, var_uv.y * aspect_scale + 0.5 - 0.5 * aspect_scale);

    // Sample texture at position, or return black if our UV is out of bounds
    vec3 texture_sample = sample_in_texture(sample_uv) ? texture(blit_tex, sample_uv).xyz : vec3(0.0);
    out_frag_color = vec4(texture_sample, 1.0);
}

#endif
