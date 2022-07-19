layout (std140) uniform GlobalParams
{
    float sim_time;
    mat4 mat_proj;
    mat4 mat_view;
};

layout (std140) uniform ModelParams
{
    mat4 mat_model;
    mat3 mat_normal;
};

layout (std140) uniform MaterialParams
{
    bool has_base_color_texture;
};
