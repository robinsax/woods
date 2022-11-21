use js_sys::{Float32Array, Uint16Array};
use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject, WebGlBuffer};

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
