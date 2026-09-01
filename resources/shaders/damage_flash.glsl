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

// The flash intensity, decaying from 1 to 0 after the player takes damage
uniform float flash_intensity;

in vec2 var_uv;

out vec4 out_frag_color;

// How strong the flash is at full intensity
const float FLASH_STRENGTH = 0.35;

void main() {
    // Subtle red tint across the whole screen
    out_frag_color = vec4(0.8, 0.05, 0.05, flash_intensity * FLASH_STRENGTH);
}

#endif
