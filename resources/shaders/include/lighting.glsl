#ifndef LIGHTING_GLSL
#define LIGHTING_GLSL

#include resources/shaders/include/uniforms.glsl

// Calculate the lighting for the given vertex/fragment
vec3 calculate_lighting(vec3 world_pos, vec3 world_normal) {
    vec3 light = vec3(0.0);

    // Calculate lighting
    for (int i = 0; i < LIGHT_COUNT; ++i) {
        if (!lights[i].enabled)
            continue;

        if (lights[i].light_type == DIRECTIONAL_LIGHT) {
            vec3 light_dir = normalize(-lights[i].light_dir);
            float diffuse = max(dot(world_normal, light_dir), 0.0);

            light += diffuse * lights[i].color * lights[i].intensity;
        }
        else if (lights[i].light_type == POINT_LIGHT) {
            const float POINT_LIGHT_INTENSITY_SCALE = 0.1;

            vec3 light_offset = lights[i].light_pos - world_pos;
            float light_dist = length(light_offset);
            vec3 light_dir = light_offset / light_dist;

            float diffuse = max(dot(world_normal, light_dir), 0.0);

            float dist_over_range = (light_dist / lights[i].range);
            float dist_over_range_4 = dist_over_range * dist_over_range * dist_over_range * dist_over_range;
            float attenuation = max(min(1.0 - dist_over_range_4, 1.0), 0.0) / (light_dist * light_dist);

            light += diffuse * lights[i].color * attenuation * lights[i].intensity * POINT_LIGHT_INTENSITY_SCALE;
        }
    }

    return light;
}

#endif
