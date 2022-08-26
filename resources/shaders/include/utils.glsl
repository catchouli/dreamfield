#ifndef UTILS_GLSL
#define UTILS_GLSL

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
const int dither_values[16] = int[16](
    -4,  0, -3,  1,
     2, -2,  3, -1,
    -3,  1, -4,  0,
     3, -1,  2, -2
);

// The first quarter of the color ramp goes up to about half intensity, and then the rest goes up more slowly
float remap_intensity_f(float color) {
    //float a = clamp(color, 0.0, 0.25) * 2.0 + clamp(color - 0.25, 0.0, 0.75) / 3.0 * 2.0;
    //float b = clamp(color - 0.25, 0.0, 0.75) / 0.75
    float a = clamp(color / 0.25, 0.0, 1.0);
    float b = clamp((color - 0.25) / 0.75, 0.0, 1.0);
    return 0.5 * (a + b);
}

// Remap the intensity to the playstation values
// https://psx-spx.consoledev.net/graphicsprocessingunitgpu/#rgb-intensity-notes
vec3 remap_intensity(vec3 color) {
    return vec3(remap_intensity_f(color.r), remap_intensity_f(color.g), remap_intensity_f(color.b));
}

// Apply the playstation dithering
// https://psx-spx.consoledev.net/graphicsprocessingunitgpu/
vec3 dither(vec3 color, ivec2 pixel_pos) {
    // Convert to integer
    ivec3 icolor = ivec3(remap_intensity(color) * 256.0);

    // Apply dithering
    ivec2 dither_index = pixel_pos % 4;
    icolor += dither_values[dither_index.y * 4 + dither_index.x];

    // Clamp to 0..256
    icolor = clamp(icolor, ivec3(0), ivec3(256));

    // Downsample to 5-bit
    icolor = icolor / 8;

    // Convert back to float
    return vec3(icolor) / 32.0;
}

#endif
