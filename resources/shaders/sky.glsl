#version 330 core

#include resources/shaders/include/constants.glsl
#include resources/shaders/include/uniforms.glsl
#include resources/shaders/include/utils.glsl

#ifdef BUILDING_VERTEX_SHADER

layout (location = 0) in vec3 vs_pos;

void main() {
    gl_Position = vec4(vs_pos.x, vs_pos.y, vs_pos.z, 1.0);
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_skybox;

out vec4 out_frag_color;

// Calculate a ray direction from a -1..1 pixel position
vec3 calc_ray_dir(vec2 pixel_pos) {
    float aspect = render_res.x / render_res.y;

    // Calculate ray direction
    vec3 ray_dir = vec3(
        pixel_pos.x * tan(render_fov * 0.5) * aspect,
        pixel_pos.y * tan(render_fov * 0.5),
        -1.0
    );

    // Apply camera rotation
    ray_dir = mat3(mat_view_inv) * ray_dir;

    return normalize(ray_dir);
}

// Project a ray direction to equirectangular texture coordinates
vec2 project_equirectangular(vec3 ray_dir) {
    return vec2(atan(ray_dir.z, ray_dir.x) + M_PI, acos(ray_dir.y)) / vec2(2.0 * M_PI, M_PI);
}

void main() {
    // Map pixel pos from -1 to 1
    vec2 pixel_pos = 2.0 * gl_FragCoord.xy / render_res - vec2(1.0);

    // Jiggle
    const float JIGGLE_PIXELS = 0.25;
    float jiggle_amount = (mod(mat_view[3][0], 1.0) + mod(mat_view[3][1], 1.0) + mod(mat_view[3][2], 1.0)) / 3.0;
    float jiggle_amount_norm = 2.0 * jiggle_amount - 1.0;
    pixel_pos.x += jiggle_amount_norm / render_res.x * JIGGLE_PIXELS;
    pixel_pos.y += jiggle_amount_norm / render_res.y * JIGGLE_PIXELS;

    // Calculate ray
    vec3 ray_dir = calc_ray_dir(pixel_pos);

    // Project ray to equirectangular
    vec2 uv = project_equirectangular(ray_dir); 

    // Sample skybox texture
    vec3 out_color = texture(tex_skybox, uv).rgb;

    // Add dithering
    const float DITHER_EXPONENT = 0.5;
    float dither_strength = pow(luma(out_color), DITHER_EXPONENT);
    out_color = dither(out_color, ivec2(gl_FragCoord.xy), dither_strength);

    out_frag_color = vec4(out_color, 1.0);
}

#endif
