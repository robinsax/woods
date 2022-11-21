use js_sys::{Float32Array, Uint16Array};
use web_sys::{
    WebGl2RenderingContext, WebGlTexture, WebGlVertexArrayObject, WebGlUniformLocation, WebGlBuffer,
    WebGlProgram
};

use super::renderer::Renderer;

pub struct Buffer {
    renderer: &'static Renderer,
    buffer_type: u32,
    buffer: WebGlBuffer
}

impl Buffer {
    pub fn new_array_buffer(renderer: &'static Renderer, vertices: Box<[f32]>) -> Self {
        let context = renderer.context();
        let buffer = context.create_buffer().unwrap();

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let view = Float32Array::view(vertices.as_ref());
    
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &view,
                WebGl2RenderingContext::STATIC_DRAW
            );
        }

        Self {
            renderer,
            buffer_type: WebGl2RenderingContext::ARRAY_BUFFER,
            buffer
        }
    }

    pub fn new_element_buffer(renderer: &'static Renderer, indices: Box<[u16]>) -> Self {
        let context = renderer.context();
        let buffer = context.create_buffer().unwrap();

        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let view = Uint16Array::view(indices.as_ref());
    
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &view,
                WebGl2RenderingContext::STATIC_DRAW
            );
        }

        Self {
            renderer,
            buffer_type: WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            buffer
        }
    }

    pub fn bind(&self) {
        self.renderer.context().bind_buffer(self.buffer_type, Some(&self.buffer));
    }

    pub fn unbind(&self) {
        self.renderer.context().bind_buffer(self.buffer_type, None);
    }
}

pub struct Vao {
    renderer: &'static Renderer,
    object: WebGlVertexArrayObject
}

impl Vao {
    pub fn new(renderer: &'static Renderer) -> Self {
        let context = renderer.context();
        let object = context.create_vertex_array().unwrap();

        Self {
            renderer,
            object
        }
    }

    pub fn bind_attrib_i32(&self, location: u32, components: i32) {
        let context = self.renderer.context();

        context.vertex_attrib_pointer_with_i32(
            location,
            components,
            WebGl2RenderingContext::FLOAT,
            false, 0, 0
        );
        context.enable_vertex_attrib_array(location);
    }

    pub fn bind(&self) {
        self.renderer.context().bind_vertex_array(Some(&self.object));
    }

    pub fn unbind(&self) {
        self.renderer.context().bind_vertex_array(None);
    }
}

pub struct Tex2d {
    renderer: &'static Renderer,
    texture: WebGlTexture
}

impl Tex2d {
    pub fn from_mem(renderer: &'static Renderer, texture_loc: &[u8]) -> Self {
        let context = renderer.context();

        let texture = context.create_texture().unwrap();

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        let texture_img = image::load_from_memory(texture_loc)
            .expect("parse image");
        let texture_data = texture_img.into_rgba8().to_vec();

        context
            // Nice!
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D, 0,
                WebGl2RenderingContext::RGBA as i32,
                16, 16, 0,
                WebGl2RenderingContext::RGBA, WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(&texture_data)
            )
            .expect("image load to gpu");

        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32
        );
        context.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 1);
        context.pixel_storei(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);

        Self {
            renderer,
            texture
        }
    }

    pub fn bind(&self, which: u32, onto: WebGlUniformLocation) {
        let context = self.renderer.context();

        context.active_texture(WebGl2RenderingContext::TEXTURE0 + which);

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));

        context.uniform1i(Some(&onto), 0);
    }

    pub fn unbind(&self) {
        self.renderer.context().bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }
}

macro_rules! compile_shader {
    ($c: ident, $s: ident, $($t: tt)*) => {
        {
            let shader = $c.create_shader($($t)*).unwrap();

            $c.shader_source(&shader, $s);
            $c.compile_shader(&shader);

            let success = $c
                .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
                .as_bool()
                .expect("compile status read");

            if !success {
                let log = $c.get_shader_info_log(&shader).unwrap();
                panic!("shader compile failed: {}", log);
            }

            shader
        }
    }
}

pub struct ShaderProgram {
    renderer: &'static Renderer,
    program: WebGlProgram
}

impl ShaderProgram {
    pub fn compile(renderer: &'static Renderer, vert_src: &str, frag_src: &str) -> Self {
        let context = renderer.context();
        let vert = compile_shader!(context, vert_src, WebGl2RenderingContext::VERTEX_SHADER);
        let frag = compile_shader!(context, frag_src, WebGl2RenderingContext::FRAGMENT_SHADER);

        let program = context.create_program().unwrap();

        context.attach_shader(&program, &vert);
        context.attach_shader(&program, &frag);
        context.link_program(&program);

        let success = context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .expect("link status read");

        if !success {
            panic!("shader link failed");
        }

        Self {
            renderer,
            program
        }
    }

    pub fn activate(&self) {
        self.renderer.context().use_program(Some(&self.program));
    }

    pub fn deactivate(&self) {
        self.renderer.context().use_program(None);
    }

    pub fn attrib_location(&self, param: &str) -> u32 {
        self.renderer.context()
            .get_attrib_location(&self.program, param)
            .try_into().expect(param)
    }

    pub fn uniform_location(&self, param: &str) -> WebGlUniformLocation {
        self.renderer.context()
            .get_uniform_location(&self.program, param).expect(param)
            .try_into().expect(param)
    }

    pub fn set_uniform_mat4(&self, param: &str, data: &[f32; 16]) {
        let loc = self.uniform_location(param);
        self.renderer.context()
            .uniform_matrix4fv_with_f32_array(Some(&loc), false, data);
    }

    pub fn set_uniform_vec4(&self, param: &str, data: &[f32; 4]) {
        let loc = self.uniform_location(param);
        self.renderer.context()
            .uniform4fv_with_f32_array(Some(&loc), data);
    }
}
