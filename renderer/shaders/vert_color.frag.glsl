#version 300 es

precision highp float;

in vec4 p_color;

out vec4 o_color;

void main() {
    o_color = p_color;
}