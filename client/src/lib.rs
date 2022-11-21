extern crate console_error_panic_hook;

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use log::{Level, debug, info};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use js_sys::Date;
use web_sys::{DedicatedWorkerGlobalScope, WebSocket, MessageEvent, BroadcastChannel};
use serde_json;

use common::{
    js_fn_into, js_fn, js_fn_leak, global_scope, init_console_logging,
    message_event_to_runtime_message, block_pattern, message_event_to
};
use common::runtime::{Runtime, RuntimeMessage, RuntimeRole, RuntimeIo};
use common::input::Input;

const COMBINED_TICK_INTERVAL_MS: i32 = 15;

struct WorkerRuntimeIoImpl {
    chan: BroadcastChannel,
    renderer_chan: BroadcastChannel,
    socket: WebSocket,
    input_rx_queue: Option<Vec<Input>>,
    message_rx_queue: Option<Vec<RuntimeMessage>>
}

impl WorkerRuntimeIoImpl {
    async fn new_static() -> &'static RefCell<Self> {
        let socket = WebSocket::new("ws://localhost/ws").expect("websocket open");

        block_pattern!(|r| socket.set_onopen(Some(&r))).await.unwrap();

        let instance = Box::leak(Box::new(RefCell::new(Self {
            chan: BroadcastChannel::new("woods").expect("chan open fail"),
            renderer_chan: BroadcastChannel::new("woods-renderer").expect("chan open fail"),
            socket,
            input_rx_queue: Some(Vec::new()),
            message_rx_queue: Some(Vec::new())
        })));

        let handle_master_message = js_fn!(|event: MessageEvent| {
            let message = message_event_to_runtime_message!(event);

            instance
                .try_borrow_mut().expect("on handle_master_message")
                .message_rx_queue
                .as_mut().expect("message_rx_queue unset")
                .push(message);
        });

        instance
            .borrow_mut()
            .socket
            .add_event_listener_with_callback("message".into(), js_fn_into!(handle_master_message))
            .unwrap();

        js_fn_leak!(handle_master_message);

        let keep_alive = js_fn!(|| {
            info!("ka ping tx");

            instance
                .try_borrow_mut().expect("on keep_alive")
                .socket
                .send_with_str("ping")
                .expect("keep_alive tx");
        });

        global_scope!(DedicatedWorkerGlobalScope)
            .set_interval_with_callback_and_timeout_and_arguments_0(
                js_fn_into!(keep_alive), 500
            )
            .unwrap();

        js_fn_leak!(keep_alive);

        let handle_input = js_fn!(|event: MessageEvent| {
            let input = message_event_to!(event, Input);

            instance
                .try_borrow_mut().expect("on handle_input")
                .input_rx_queue
                .as_mut().expect("input_rx_queue unset")
                .push(input);
        });

        instance
            .borrow_mut()
            .chan
            .add_event_listener_with_callback("message".into(), js_fn_into!(handle_input))
            .unwrap();
        
        js_fn_leak!(handle_input);

        instance
    }

    fn rx(&mut self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        let inputs = self.input_rx_queue.take().expect("input_rx_queue unset");
        let messages = self.message_rx_queue.take().expect("message_rx_queue unset");

        self.input_rx_queue = Some(Vec::new());
        self.message_rx_queue = Some(Vec::new());

        (inputs, messages)
    }

    fn tx(&mut self, message: RuntimeMessage, explicit_down: bool) {
        let serialized = serde_json::to_string(&message).expect("serialize message failed");
        debug!("tx {:?} {}", serialized, explicit_down);

        if explicit_down {
            self.renderer_chan.post_message(&serialized.into()).expect("message chan tx");
        }
        else {
            self.socket.send_with_str(serialized.as_str()).expect("message socket tx");
        }
    }
}

struct WorkerRuntimeIo {
    inner: &'static RefCell<WorkerRuntimeIoImpl>
}

// SAFETY: There is one thread.
unsafe impl Send for WorkerRuntimeIo {}
unsafe impl Sync for WorkerRuntimeIo {}

impl RuntimeIo for WorkerRuntimeIo {
    fn rx(&self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        self.inner.try_borrow_mut().expect("rx inputs").rx()
    }

    fn tx(&self, message: RuntimeMessage, explicit_down: bool) {
        self.inner.try_borrow_mut().expect("tx updates").tx(message, explicit_down)
    }
}

impl WorkerRuntimeIo {
    async fn new_static() -> &'static Self {
        Box::leak(Box::new(Self {
            inner: WorkerRuntimeIoImpl::new_static().await
        }))
    }
}

#[wasm_bindgen]
pub fn main() {
    spawn_local(async {
        init_console_logging!();

        info!("boot client");

        let global = global_scope!(DedicatedWorkerGlobalScope);

        let runtime_io = WorkerRuntimeIo::new_static().await;
        let runtime = Runtime::new_static_cell(runtime_io, RuntimeRole::Intermediate);

        let mut last_t = Date::now();
        let handle_tick = js_fn!(move || {
            let mut rt_lock = runtime.borrow_mut();

            let cur_t = Date::now();
            rt_lock.systems_tick((cur_t - last_t) / 1000.0);
            last_t = cur_t;

            rt_lock.io_tick();
        });

        global
            .set_interval_with_callback_and_timeout_and_arguments_0(
                js_fn_into!(handle_tick), COMBINED_TICK_INTERVAL_MS
            )
            .unwrap();

        js_fn_leak!(handle_tick);

        info!("loop inited");
    });
}
