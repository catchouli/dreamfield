#ifndef UNIFORMS_GLSL
#define UNIFORMS_GLSL

// Global
layout (std140) uniform GlobalParams
{
    mat4 mat_proj;
    mat4 mat_view;
    mat4 mat_view_inv;
    mat4 mat_view_proj;
    mat4 mat_view_proj_inv;
    mat4 mat_model;
    mat4 mat_model_view_proj;
    mat3 mat_normal;

    float sim_time;
    float target_aspect;
    float window_aspect;
    float render_fov;
    vec2 render_res;

    vec3 fog_color;
    vec2 fog_dist;

    float lighting_strength;
};

// Materials
layout (std140) uniform MaterialParams
{
    bool has_base_color_texture;
    vec4 base_color;
};

// Lights
#define LIGHT_COUNT 20
#define POINT_LIGHT 0
#define DIRECTIONAL_LIGHT 1
#define SPOT_LIGHT 2

struct Light
{
    bool enabled;
    int light_type;

    float intensity;
    float range;

    float inner_cone_angle;
    float outer_cone_angle;

    vec3 color;
    vec3 light_dir;
    vec3 light_pos;
};

layout (std140) uniform LightParams
{
    vec3 ambient_light;
    Light lights[LIGHT_COUNT];
};

// Joints
#define JOINT_COUNT 30

struct Joint
{
    mat4 joint_matrix;
};

layout (std140) uniform JointParams
{
    bool skinning_enabled;
    Joint joints[JOINT_COUNT];
};

#endif
