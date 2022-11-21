use web_sys::{WebGl2RenderingContext, WebGlUniformLocation, WebGlProgram};

use super::renderer::Renderer;

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

    pub fn use_program(&self) {
        self.renderer.context().use_program(Some(&self.program));
    }

    pub fn attrib_location(&self, param: &str) -> u32 {
        self.renderer.context().get_attrib_location(&self.program, param)
            .try_into().expect("signed attribute location")
    }

    pub fn uniform_location(&self, param: &str) -> WebGlUniformLocation {
        self.renderer.context().get_uniform_location(&self.program, param).unwrap()
            .try_into().expect("signed attribute location")
    }
}
