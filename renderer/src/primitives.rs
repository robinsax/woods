use web_sys::WebGl2RenderingContext;

use super::mem::{Vao, Buffer};
use super::shader::ShaderProgram;
use super::renderer::Renderer;

pub struct RenderPrimitive {
    renderer: &'static Renderer,
    shader: ShaderProgram,
    indices: i32,
    vao: Vao
}

impl RenderPrimitive {
    pub fn activate(&self, view_mat: &[f32; 16], proj_mat: &[f32; 16]) {
        let context = self.renderer.context();

        let view_mat_loc = self.shader.uniform_location("u_view");
        let proj_mat_loc = self.shader.uniform_location("u_projection");

        self.vao.bind();
        self.shader.use_program();

        context.uniform_matrix4fv_with_f32_array(Some(&proj_mat_loc), false, proj_mat);
        context.uniform_matrix4fv_with_f32_array(Some(&view_mat_loc), false, view_mat);
    }

    pub fn deactivate(&self) {
        self.vao.unbind();
    }

    pub fn draw(&self, model_mat: &[f32; 16]) {
        let model_mat_loc = self.shader.uniform_location("u_model");

        let context = self.renderer.context();

        context.uniform_matrix4fv_with_f32_array(Some(&model_mat_loc), false, model_mat);

        context.draw_elements_with_f64(
            WebGl2RenderingContext::TRIANGLES,
            self.indices,
            WebGl2RenderingContext::UNSIGNED_SHORT, 0.0
        );
    }
}

pub fn cube_primitive(renderer: &'static Renderer) -> RenderPrimitive {
    let shader = ShaderProgram::compile(
        renderer,
        include_str!("../shaders/mvp.vert.glsl"),
        include_str!("../shaders/vert_color.frag.glsl")
    );

    shader.use_program();

    let vertices: [f32; 72] = [
        // Front face
        -1.0, -1.0,  1.0,
        1.0, -1.0,  1.0,
        1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,

        // Back face
        -1.0, -1.0, -1.0,
        -1.0,  1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0, -1.0, -1.0,

        // Top face
        -1.0,  1.0, -1.0,
        -1.0,  1.0,  1.0,
        1.0,  1.0,  1.0,
        1.0,  1.0, -1.0,

        // Bottom face
        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0, -1.0,  1.0,
        -1.0, -1.0,  1.0,

        // Right face
        1.0, -1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0,  1.0,  1.0,
        1.0, -1.0,  1.0,

        // Left face
        -1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0,
        -1.0,  1.0, -1.0,
    ];
    let color_src = [
        [1.0,  1.0,  1.0,  1.0],    // Front face: white
        [1.0,  0.0,  0.0,  1.0],    // Back face: red
        [0.0,  1.0,  0.0,  1.0],    // Top face: green
        [0.0,  0.0,  1.0,  1.0],    // Bottom face: blue
        [1.0,  1.0,  0.0,  1.0],    // Right face: yellow
        [1.0,  0.0,  1.0,  1.0],    // Left face: purple
    ];
    let mut colors = Vec::new();

    for row in color_src {
        for _ in 0..4 {
            colors.push(row[0]);
            colors.push(row[1]);
            colors.push(row[2]);
            colors.push(row[3]);
        }
    }

    let indices: [u16; 36] = [
        0,  1,  2,      0,  2,  3,    // front
        4,  5,  6,      4,  6,  7,    // back
        8,  9,  10,     8,  10, 11,   // top
        12, 13, 14,     12, 14, 15,   // bottom
        16, 17, 18,     16, 18, 19,   // right
        20, 21, 22,     20, 22, 23,   // left
    ];

    // Create and bind VAO.
    let vao = Vao::new(renderer);
    vao.bind();

    // Bind elem buffer (implicit attach).
    let elem_buf = Buffer::new_element_buffer(&renderer, Box::new(indices));

    // Bind vert buffer (explicit attach).
    let position_loc = shader.attrib_location("i_position");
    let vert_buf = Buffer::new_array_buffer(&renderer, Box::new(vertices));
    vao.bind_attrib_i32(position_loc, 3);

    // Bind vert buffer (explicit attach).
    let color_loc = shader.attrib_location("i_color");
    let colors_slice: [f32; 96] = colors.try_into().unwrap();
    let color_buf = Buffer::new_array_buffer(&renderer, Box::new(colors_slice));
    vao.bind_attrib_i32(color_loc, 4);

    // Done.
    vao.unbind();
    elem_buf.unbind();
    vert_buf.unbind();
    color_buf.unbind();

    RenderPrimitive {
        vao,
        shader,
        renderer,
        indices: indices.len().try_into().unwrap()
    }
}

pub fn quad_primitive(renderer: &'static Renderer) -> RenderPrimitive {
    let shader = ShaderProgram::compile(
        renderer,
        include_str!("../shaders/mvp.vert.glsl"),
        include_str!("../shaders/flat_color.frag.glsl")
    );

    shader.use_program();

    let color_loc = shader.uniform_location("u_color");
    let color: [f32; 4] = [1.0, 0.6, 0.6, 1.0];
    renderer.context().uniform4fv_with_f32_array(Some(&color_loc), &color);

    let vertices: [f32; 12] = [
        -1.0, -1.0,  0.0,
        1.0, -1.0,  0.0,
        1.0,  1.0,  0.0,
        -1.0,  1.0,  0.0
    ];
    let indices: [u16; 6] = [
        0,  1,  2, 0,  2,  3
    ];

    // Create and bind VAO.
    let vao = Vao::new(renderer);
    vao.bind();

    // Bind elem buffer (implicit attach).
    let elem_buf = Buffer::new_element_buffer(&renderer, Box::new(indices));

    // Bind vert buffer (explicit attach).
    let position_loc = shader.attrib_location("i_position");
    let vert_buf = Buffer::new_array_buffer(&renderer, Box::new(vertices));
    vao.bind_attrib_i32(position_loc, 3);

    // Done.
    vao.unbind();
    elem_buf.unbind();
    vert_buf.unbind();

    RenderPrimitive {
        vao,
        renderer,
        shader,
        indices: indices.len().try_into().unwrap()
    }
}