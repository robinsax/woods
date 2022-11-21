extern crate console_error_panic_hook;

use common::components::BodyComponent;
use wasm_bindgen::prelude::*;

use log::{Level, info};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::Window;

use common::{js_fn_into, js_fn, js_fn_leak, global_scope, init_console_logging, block_pattern};
use common::runtime::{Runtime, RuntimeRole};

mod renderer;
mod proxies;
mod io;
mod primitives;

use self::io::RendererRuntimeIo;
use self::renderer::Renderer;

const IO_TICK_INTERVAL_MS: i32 = 15;

#[wasm_bindgen]
pub fn main() {
    init_console_logging!();

    info!("boot renderer");

    let global = global_scope!(Window);

    let runtime = Runtime::new_static_cell(RendererRuntimeIo::new_static(), RuntimeRole::Renderer);
    let renderer: &'static Renderer = Box::leak(Box::new(Renderer::new_static_attached_to("#canvas")));

    let handle_tick = js_fn!(|| {
        let mut runtime_lock = runtime.try_borrow_mut().expect("on tick");

        runtime_lock.io_tick();
    });

    global
        .set_interval_with_callback_and_timeout_and_arguments_0(
            js_fn_into!(handle_tick), IO_TICK_INTERVAL_MS
        )
        .unwrap();

    js_fn_leak!(handle_tick);

    info!("renderer loops inited");

    spawn_local(async {
        loop {
            block_pattern!(|r| global_scope!(Window).request_animation_frame(&r).unwrap()).await.unwrap();

            let runtime_borrow = runtime
                .try_borrow().expect("render tick");
            let bodies: Vec<&BodyComponent> = runtime_borrow
                .ecs()
                .get_components::<BodyComponent>()
                .into_iter()
                .map(|(_, body)| body)
                .collect();

            renderer.render(bodies);
        }
    });
}
