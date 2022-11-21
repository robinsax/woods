#version 300 es

precision highp float;

in vec2 p_uv;

uniform sampler2D u_tex;
uniform sampler2D u_normals;
uniform vec3 u_sun_direction;

out vec4 o_color;

void main() {
    vec4 sample = texture(u_tex, p_uv);
    float alpha = sample.w;
    vec3 color = sample.xyz;

    // TODO: Use pre-normalized textures.
    vec3 normal = normalize(texture(u_normals, p_uv).xyz);

    o_color = vec4(color.x, alpha);
}
