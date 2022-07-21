#version 330 core

#include resources/shaders/include/uniforms.glsl
#define M_PI 3.1415926535897932384626433832795

#ifdef BUILDING_VERTEX_SHADER

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_nrm;
layout (location = 3) in vec2 in_uv;

out vec3 var_pos;
out vec3 var_nrm;
out vec2 var_uv;

void main() {
    mat4 mvp = mat_proj * mat_view * mat_model;

    var_pos = (mat_model * vec4(in_pos, 1.0)).xyz;
    var_nrm = normalize(mat_normal * in_nrm);
    var_uv = in_uv;
    gl_Position = mat_model_view_proj * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#endif

#ifdef BUILDING_FRAGMENT_SHADER

uniform sampler2D tex_base_color;

in vec3 var_pos;
in vec3 var_nrm;
in vec2 var_uv;

out vec4 out_frag_color;

void main() {
    vec3 sun_dir = normalize(vec3(0.5, 0.5, 0.5));
    vec3 base_color = has_base_color_texture ? texture(tex_base_color, var_uv).xyz : vec3(1.0);
    float diffuse_strength = dot(sun_dir, var_nrm);

    vec3 light = vec3(0.0);

    // Calculate lighting
    for (int i = 0; i < LIGHT_COUNT; ++i) {
        if (!lights[i].enabled)
            continue;

        if (lights[i].light_type == DIRECTIONAL_LIGHT) {
            vec3 light_dir = normalize(-lights[i].light_dir);
            float diffuse = max(dot(var_nrm, light_dir), 0.0);

            light += diffuse * lights[i].color * lights[i].intensity;
        }
        else if (lights[i].light_type == POINT_LIGHT) {
            const float POINT_LIGHT_INTENSITY_SCALE = 0.1;

            vec3 light_dir = normalize(lights[i].light_pos - var_pos);

            float diffuse = max(dot(var_nrm, light_dir), 0.0);

            float light_dist = length(var_pos - lights[i].light_pos);

            float dist_over_range = (light_dist / lights[i].range);
            float dist_over_range_4 = dist_over_range * dist_over_range * dist_over_range * dist_over_range;
            float attenuation = max(min(1.0 - dist_over_range_4, 1.0), 0.0) / (light_dist * light_dist);

            light += diffuse * lights[i].color * attenuation * lights[i].intensity * POINT_LIGHT_INTENSITY_SCALE;
        }
    }

    vec3 out_color = light * base_color;
    out_color = min(out_color, vec3(1.0));

    out_frag_color = vec4(out_color, 1.0);
}

#endif
