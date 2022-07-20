layout (std140) uniform GlobalParams
{
    mat4 mat_proj;
    mat4 mat_view;
    mat4 mat_view_proj;
    mat4 mat_view_proj_inv;
    mat4 mat_model;
    mat4 mat_model_view_proj;
    mat3 mat_normal;
    float sim_time;
    float vp_aspect;
};

layout (std140) uniform MaterialParams
{
    bool has_base_color_texture;
};
