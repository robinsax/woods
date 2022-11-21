#version 300 es
 
in vec3 i_position;
in vec4 i_color;
in vec2 i_uv;

uniform mat4 u_view;
uniform mat4 u_projection;
uniform mat4 u_model;

out vec4 p_color;
out vec2 p_uv;

void main() {
    gl_Position = u_projection * u_view * u_model * vec4(i_position, 1.0);
    p_color = i_color;
    p_uv = i_uv;
}
