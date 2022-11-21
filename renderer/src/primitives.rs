use web_sys::WebGl2RenderingContext;

use crate::proxies::Tex2d;

use super::proxies::{Vao, Buffer, ShaderProgram};
use super::renderer::Renderer;

pub struct RenderPassParams<'a> {
    pub view_mat: &'a [f32; 16],
    pub projection_mat: &'a [f32; 16],
    pub camera_pos: &'a [f32; 3]
}

pub struct DrawParams<'a> {
    pub model_mat: &'a [f32; 16],
    pub camera_pos: &'a [f32; 3],
    pub sun_direction: &'a [f32; 3]
}

pub trait RenderPrimitive {
    fn activate(&self, params: &RenderPassParams);
    fn deactivate(&self);
    fn draw(&self, params: &DrawParams);
}

pub struct Quad {
    renderer: &'static Renderer,
    shader: ShaderProgram,
    vao: Vao
}

impl Quad {
    pub fn new_flat_color(renderer: &'static Renderer, color: &[f32; 4]) -> Self {
        let shader = ShaderProgram::compile(
            renderer,
            include_str!("../shaders/mvp.vert.glsl"),
            include_str!("../shaders/flat_color.frag.glsl")
        ); 
        shader.activate();
    
        shader.set_uniform_vec4("u_color", color);

        let vertices: [f32; 12] = [
            -1.0, -1.0,  0.0,
            1.0, -1.0,  0.0,
            1.0,  1.0,  0.0,
            -1.0,  1.0,  0.0
        ];
        let indices: [u16; 6] = [
            0,  1,  2, 0,  2,  3
        ];

        let vao = Vao::new(renderer);
        vao.bind();
    
        let elem_buf = Buffer::new_element_buffer(&renderer, Box::new(indices));

        let vert_buf = Buffer::new_array_buffer(&renderer, Box::new(vertices));
        vao.bind_attrib_i32(shader.attrib_location("i_position"), 3);
    
        vao.unbind();
        elem_buf.unbind();
        vert_buf.unbind();
    
        Self {
            vao,
            renderer,
            shader
        }
    }
}

impl RenderPrimitive for Quad {
    fn activate(&self, params: &RenderPassParams) {
        self.vao.bind();
        self.shader.activate();

        self.shader.set_uniform_mat4("u_projection", params.projection_mat);
        self.shader.set_uniform_mat4("u_view", params.view_mat);
    }

    fn deactivate(&self) {
        self.vao.unbind();
        self.shader.deactivate();
    }

    fn draw(&self, params: &DrawParams) {
        let context = self.renderer.context();

        self.shader.set_uniform_mat4("u_model", params.model_mat);

        context.draw_elements_with_f64(
            WebGl2RenderingContext::TRIANGLES,
            6,
            WebGl2RenderingContext::UNSIGNED_SHORT, 0.0
        );
    }
}

pub struct Cube {
    renderer: &'static Renderer,
    shader: ShaderProgram,
    vao: Vao
}

impl Cube {
    pub fn new_face_colored(renderer: &'static Renderer, face_colors: &[[f32; 4]; 6]) -> Self {
        let shader = ShaderProgram::compile(
            renderer,
            include_str!("../shaders/mvp.vert.glsl"),
            include_str!("../shaders/vert_color.frag.glsl")
        );
        shader.activate();
    
        let vertices: [f32; 72] = [
            -1.0, -1.0,  1.0,
            1.0, -1.0,  1.0,
            1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
    
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            1.0,  1.0, -1.0,
            1.0, -1.0, -1.0,
    
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            1.0,  1.0,  1.0,
            1.0,  1.0, -1.0,
    
            -1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,
    
            1.0, -1.0, -1.0,
            1.0,  1.0, -1.0,
            1.0,  1.0,  1.0,
            1.0, -1.0,  1.0,
    
            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0
        ];

        let indices: [u16; 36] = [
            0, 1, 2, 0, 2, 3,
            4, 5, 6, 4, 6, 7,
            8, 9, 10, 8, 10, 11,
            12, 13, 14, 12, 14, 15,
            16, 17, 18, 16, 18, 19,
            20, 21, 22, 20, 22, 23
        ];

        let mut color_vec = Vec::new();    
        for face in face_colors {
            for _ in 0..4 {
                color_vec.push(face[0]);
                color_vec.push(face[1]);
                color_vec.push(face[2]);
                color_vec.push(face[3]);
            }
        }
    
        let vao = Vao::new(renderer);
        vao.bind();
    
        let elem_buf = Buffer::new_element_buffer(&renderer, Box::new(indices));

        let vert_buf = Buffer::new_array_buffer(&renderer, Box::new(vertices));
        vao.bind_attrib_i32(shader.attrib_location("i_position"), 3);
    
        let colors: [f32; 96] = color_vec.try_into().unwrap();
        let color_buf = Buffer::new_array_buffer(&renderer, Box::new(colors));
        vao.bind_attrib_i32(shader.attrib_location("i_color"), 4);
    
        vao.unbind();
        elem_buf.unbind();
        vert_buf.unbind();
        color_buf.unbind();    
    
        Self {
            vao,
            renderer,
            shader
        }
    }
}

impl RenderPrimitive for Cube {
    fn activate(&self, params: &RenderPassParams) {
        self.vao.bind();
        self.shader.activate();

        self.shader.set_uniform_mat4("u_projection", params.projection_mat);
        self.shader.set_uniform_mat4("u_view", params.view_mat);
    }

    fn deactivate(&self) {
        self.vao.unbind();
        self.shader.deactivate();
    }

    fn draw(&self, params: &DrawParams) {
        let context = self.renderer.context();

        self.shader.set_uniform_mat4("u_model", params.model_mat);

        context.draw_elements_with_f64(
            WebGl2RenderingContext::TRIANGLES,
            36,
            WebGl2RenderingContext::UNSIGNED_SHORT, 0.0
        );
    }
}

pub struct Tex2dQuad {
    renderer: &'static Renderer,
    shader: ShaderProgram,
    texture: Tex2d,
    vao: Vao
}

impl Tex2dQuad {
    pub fn new_from_mem(renderer: &'static Renderer, texture_loc: &[u8]) -> Self {
        let shader = ShaderProgram::compile(
            renderer,
            include_str!("../shaders/mvp.vert.glsl"),
            include_str!("../shaders/tex_2d.frag.glsl")
        ); 
        shader.activate();

        let vertices: [f32; 12] = [
            0.0, -1.0,  -1.0,
            0.0, -1.0,  1.0,
            0.0,  1.0,  1.0,
            0.0,  1.0,  -1.0
        ];
        let indices: [u16; 6] = [
            0,  1,  2, 0,  2,  3
        ];
        let uvs: [f32; 8] = [
            0.0,  1.0,
            0.0,  0.0,
            1.0,  0.0,
            1.0,  1.0
        ];

        let vao = Vao::new(renderer);
        vao.bind();
    
        let elem_buf = Buffer::new_element_buffer(&renderer, Box::new(indices));

        let vert_buf = Buffer::new_array_buffer(&renderer, Box::new(vertices));
        vao.bind_attrib_i32(shader.attrib_location("i_position"), 3);

        let uv_buf = Buffer::new_array_buffer(&renderer, Box::new(uvs));
        vao.bind_attrib_i32(shader.attrib_location("i_uv"), 2);
    
        let texture = Tex2d::from_mem(renderer, texture_loc);

        vao.unbind();
        elem_buf.unbind();
        vert_buf.unbind();
        uv_buf.unbind();

        Self {
            vao,
            texture,
            renderer,
            shader
        }
    }
}

impl RenderPrimitive for Tex2dQuad {
    fn activate(&self, params: &RenderPassParams) {
        self.vao.bind();
        self.shader.activate();
        self.texture.bind(0, self.shader.uniform_location("u_tex"));

        self.shader.set_uniform_mat4("u_projection", params.projection_mat);
        self.shader.set_uniform_mat4("u_view", params.view_mat);
    }

    fn deactivate(&self) {
        self.vao.unbind();
        self.texture.unbind();
        self.shader.deactivate();
    }

    fn draw(&self, params: &DrawParams) {
        let context = self.renderer.context();

        let theta = (
            (params.model_mat[12] - params.camera_pos[0]) / 
            (params.model_mat[13] - params.camera_pos[1])
        ).atan() + (std::f32::consts::PI / 2.0);

        let mut rotated_model_mat = params.model_mat.clone();
        rotated_model_mat[0] = theta.cos();
        rotated_model_mat[1] = -theta.sin();
        rotated_model_mat[4] = theta.sin();
        rotated_model_mat[5] = theta.cos();

        self.shader.set_uniform_mat4("u_model", &rotated_model_mat);

        context.draw_elements_with_f64(
            WebGl2RenderingContext::TRIANGLES,
            6,
            WebGl2RenderingContext::UNSIGNED_SHORT, 0.0
        );
    }
}
