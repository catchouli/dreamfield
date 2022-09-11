#version 330 core

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

void main() {
    vec4 tex_sample = texture(blit_tex, var_uv);
    if (tex_sample.a < 0.1)
        discard;
    out_frag_color = vec4(tex_sample);
}

#endif
