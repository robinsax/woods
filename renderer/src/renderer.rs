use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::{Window, WebGl2RenderingContext, HtmlCanvasElement};
use cgmath::{Vector3, Point3, Matrix4, ortho};

use common::global_scope;
use common::components::BodyComponent;

use super::primitives::{RenderPrimitive, cube_primitive, quad_primitive};

pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: WebGl2RenderingContext,
    primitives: RefCell<HashMap<String, RenderPrimitive>>
}

impl Renderer {
    pub fn new_static_attached_to(selector: &str) -> &'static Self {
        let canvas = global_scope!(Window)
            .document().unwrap()
            .query_selector(selector).unwrap().expect("unmatched selector")
            .dyn_into::<HtmlCanvasElement>().expect("non-canvas");

        let context = canvas
            .get_context("webgl2").unwrap().unwrap()
            .dyn_into::<WebGl2RenderingContext>().unwrap();

        let instance = Box::leak(Box::new(Self {
            canvas,
            context,
            primitives: RefCell::new(HashMap::new())
        }));

        let cube = cube_primitive(instance);
        let quad = quad_primitive(instance);

        instance.primitives.borrow_mut().insert("cube".into(), cube);
        instance.primitives.borrow_mut().insert("quad".into(), quad);
    
        instance
    }

    pub fn viewport_size(&self) -> (f32, f32) {
        let canvas_bbox = self.canvas.get_bounding_client_rect();

        (canvas_bbox.width() as f32, canvas_bbox.height() as f32)
    }

    fn apply_sizing(&self) {
        let (f_w, f_h) = self.viewport_size();

        self.canvas.set_width(f_w as u32);
        self.canvas.set_height(f_h as u32);

        self.context.viewport(0, 0, f_w as i32, f_h as i32);
    }

    pub fn context(&self) -> &WebGl2RenderingContext {
        &self.context
    }

    pub fn render(&self, bodies: Vec<&BodyComponent>) {
        self.apply_sizing();

        self.context.clear_color(0.1, 0.1, 0.1, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
     
        let (w, h) = self.viewport_size();
        let proj_mat = ortho::<f32>(
            -w / 100.0, w / 100.0,
            -h / 100.0, h / 100.0,
            0.1, 1000.0
        );
        let proj_mat_ref: &[f32; 16] = proj_mat.as_ref();
    
        let view_mat = Matrix4::<f32>::look_at_rh(
            Point3::new(-16.0, -16.0, 10.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_z()
        );
        let view_mat_ref: &[f32; 16] = view_mat.as_ref();
   
        let primitives = self.primitives.borrow();
        let cube = primitives.get("cube").unwrap();
        let quad = primitives.get("quad").unwrap();
        
        cube.activate(view_mat_ref, proj_mat_ref);
        for body in bodies.into_iter() {
            let model_mat: [f32; 16] = [
                body.sx as f32, 0.0, 0.0, 0.0,
                0.0, body.sy as f32, 0.0, 0.0,
                0.0, 0.0, body.sz as f32, 0.0,
                body.x as f32, body.y as f32, body.z as f32, 1.0
            ];

            cube.draw(&model_mat);
        }
    
        quad.activate(view_mat_ref, proj_mat_ref);
        for k in -10..11 {
            let model_mat: [f32; 16] = [
                0.01, 0.0, 0.0, 0.0,
                0.0, 10.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                k as f32, 0.0, 0.0, 1.0
            ];
    
            quad.draw(&model_mat);
        }
        for k in -10..11 {
            let model_mat: [f32; 16] = [
                10.0, 0.0, 0.0, 0.0,
                0.0, 0.01, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, k as f32, 0.0, 1.0
            ];
    
            quad.draw(&model_mat);
        }
    }
}
