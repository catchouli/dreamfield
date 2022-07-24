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

#endif
