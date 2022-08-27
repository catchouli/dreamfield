#ifndef UTILS_GLSL
#define UTILS_GLSL

// The gamma for gamma correction
const float SRGB_GAMMA = 2.2;

// Gamma correction (srgb -> linear)
vec3 srgb_to_linear(vec3 color_srgb) {
    return pow(color_srgb, vec3(SRGB_GAMMA));
}

// Gamma correction (linear -> srgb)
vec3 linear_to_srgb(vec3 color_linear) {
    return pow(color_linear, vec3(1.0 / SRGB_GAMMA));
}


// Snaps a clip pos to a specified pixel grid
// TODO: don't know if this is right tbh
vec4 snap_pos(vec4 clip_pos, vec2 pixel_grid) {
    // Convert to cartesian coordinates
    vec2 snapped = clip_pos.xy / clip_pos.w;

    // Snap to pixel grid
    snapped = floor(pixel_grid * snapped) / pixel_grid;

    // Convert back to homogeneous coordinates
    snapped *= clip_pos.w;

    return vec4(snapped, clip_pos.z, clip_pos.w);
}

// The playstation dithering values
const float dither_values[16] = float[16](
    -4.0,  0.0, -3.0,  1.0,
     2.0, -2.0,  3.0, -1.0,
    -3.0,  1.0, -4.0,  0.0,
     3.0, -1.0,  2.0, -2.0
);

// The first quarter of the color ramp goes up to about half intensity, and then the rest goes up more slowly
float remap_intensity_f(float color) {
    float a = clamp(color / 0.25, 0.0, 1.0);
    float b = clamp((color - 0.25) / 0.75, 0.0, 1.0);
    return 0.5 * (a + b);
}

// Remap the intensity to the playstation values
// https://psx-spx.consoledev.net/graphicsprocessingunitgpu/#rgb-intensity-notes
vec3 remap_intensity(vec3 color) {
    return vec3(remap_intensity_f(color.r), remap_intensity_f(color.g), remap_intensity_f(color.b));
}

// Calculate the luma from the color
float luma(vec3 color) {
    return color.r * 0.3 + color.g * 0.59 + color.b * 0.11;
}

// Apply the playstation dithering
// https://psx-spx.consoledev.net/graphicsprocessingunitgpu/
// TODO: fix low values clamping to 0.0 and high values clamping to 1.0, if it's an issue
vec3 dither(vec3 color, ivec2 pixel_pos, float intensity) {
    // Apply dithering
    ivec2 dither_index = pixel_pos % 4;
    vec3 dither = vec3(dither_values[dither_index.y * 4 + dither_index.x] * intensity / 256.0);

    color = clamp(color + dither, 0.0, 1.0);

    return color;
}

#endif
