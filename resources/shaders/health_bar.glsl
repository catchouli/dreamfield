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

// The player's health fraction, from 0 to 1
uniform float health_fraction;

in vec2 var_uv;

out vec4 out_frag_color;

// Bar geometry as fractions of the screen, bottom-left anchored
const vec2 BAR_POS = vec2(0.02, 0.02);
const vec2 BAR_SIZE = vec2(0.3, 0.055);
const float BORDER_WIDTH = 0.004;

void main() {
    // Discard everything outside the bar's slot
    vec2 rel = var_uv - BAR_POS;
    if (rel.x < 0.0 || rel.y < 0.0 || rel.x > BAR_SIZE.x || rel.y > BAR_SIZE.y) {
        discard;
    }

    // Border, empty slot, and health fill
    bool in_border = (rel.x < BORDER_WIDTH || rel.y < BORDER_WIDTH
        || rel.x > BAR_SIZE.x - BORDER_WIDTH || rel.y > BAR_SIZE.y - BORDER_WIDTH);
    bool in_fill = (!in_border
        && rel.x < (BAR_SIZE.x - BORDER_WIDTH * 2.0) * health_fraction + BORDER_WIDTH);

    vec3 color = in_border ? vec3(0.75, 0.6, 0.3) : vec3(0.05, 0.05, 0.05);
    if (in_fill) {
        color = vec3(0.7, 0.12, 0.1);
    }

    out_frag_color = vec4(color, 1.0);
}

#endif
