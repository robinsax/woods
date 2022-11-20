extern crate console_error_panic_hook;

use std::panic;
use std::cell::RefCell;

use js_sys::Date;
use log::{Level, debug, info};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use js_sys::{global, JsString, Promise};
use web_sys::{DedicatedWorkerGlobalScope, WebSocket, MessageEvent, BroadcastChannel};
use serde_json;

use common::runtime::{Runtime, RuntimeMessage, RuntimeRole, RuntimeIo};
use common::input::Input;

macro_rules! js_fn {
    (move || { $($f: tt)* }) => {
        Closure::<dyn FnMut()>::wrap(Box::new(move || { $($f)* }))
    };
    (move |$a: ident : $at: ty| { $($f: tt)* }) => {
        Closure::<dyn FnMut($at)>::wrap(Box::new(move |$a: $at| { $($f)* }))
    };
    (|| { $($f: tt)* }) => {
        Closure::<dyn FnMut()>::wrap(Box::new(|| { $($f)* }))
    };
    (|$a: ident : $at: ty| { $($f: tt)* }) => {
        Closure::<dyn FnMut($at)>::wrap(Box::new(|$a: $at| { $($f)* }))
    };
}

macro_rules! js_fn_into {
    ($s: ident) => {
        $s.as_ref().unchecked_ref()
    };
}

macro_rules! js_fn_leak {
    ($s: ident) => {
        $s.forget()
    }
}

struct JsRuntimeIoImpl {
    chan: BroadcastChannel,
    socket: WebSocket,
    inputs: Option<Vec<Input>>,
    master_messages: Option<Vec<RuntimeMessage>>
}

macro_rules! block_pattern {
    (|$r: ident| $($b: tt)*) => {
        JsFuture::from(Promise::new(&mut |resolve, _| {
            (|$r| $($b)*)(resolve)
        }))
    };
}

impl JsRuntimeIoImpl {
    async fn new_static(glob: &DedicatedWorkerGlobalScope) -> &'static RefCell<Self> {
        let socket = WebSocket::new("ws://localhost/ws").expect("ws open fail");

        block_pattern!(|r| socket.set_onopen(Some(&r))).await.unwrap();

        let inst = Box::leak(Box::new(RefCell::new(Self {
            chan: BroadcastChannel::new("woods").expect("chan open fail"),
            socket,
            inputs: Some(Vec::new()),
            master_messages: Some(Vec::new())
        })));

        let handle_update = js_fn!(|event: MessageEvent| {
            let data = event.data().dyn_into::<JsString>().expect("ws payload cast fail");

            let message = serde_json::from_str::<RuntimeMessage>(data.as_string().unwrap().as_str()).expect("master deser fail");

            debug!("rx from master ws {:?}", message);
            match &message {
                RuntimeMessage::ComponentUpdate(..) => {},
                other => info!("rx major from ws {:?}", other)
            };

            inst.try_borrow_mut().expect("handle update").master_messages.as_mut().expect("master messages unset").push(message);
        });

        let keep_alive = js_fn!(|| {
            info!("ka ping tx");

            inst
                .try_borrow_mut().expect("ka ping")
                .socket
                .send_with_str("ping")
                .expect("ka ping tx");
        });

        glob
            .set_interval_with_callback_and_timeout_and_arguments_0(
                js_fn_into!(keep_alive), 500
            )
            .expect("ka interval set");

        js_fn_leak!(keep_alive);

        inst
            .borrow_mut()
            .socket
            .add_event_listener_with_callback("message".into(), js_fn_into!(handle_update))
            .expect("ws rx attach fail");

        js_fn_leak!(handle_update);

        let pool_input = js_fn!(|message: MessageEvent| {
            let serial = message.data().as_string().expect("non-str msg");

            let mut mut_inst = inst.try_borrow_mut().expect("inst refcell fail");

            mut_inst.socket.send_with_str(serial.as_str()).expect("ws tx fail");

            if let Ok(item) = serde_json::from_str::<Input>(serial.as_str()) {
                mut_inst.inputs.as_mut().expect("input pool unset push").push(item);
            }
            else {
                debug!("invalid input data {:?}", serial);
            }
        });

        inst
            .borrow_mut()
            .chan
            .add_event_listener_with_callback("message".into(), js_fn_into!(pool_input))
            .expect("chan rx attach fail");
        
        js_fn_leak!(pool_input);

        inst
    }

    fn rx(&mut self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        let inputs = self.inputs.take().expect("input pool unset rx");
        let master_messages = self.master_messages.take().expect("master messages unset rx");

        self.inputs = Some(Vec::new());
        self.master_messages = Some(Vec::new());

        (inputs, master_messages)
    }

    fn tx(&mut self, message: RuntimeMessage) {
        let serialized = serde_json::to_string(&message).expect("des failed");
        debug!("tx to master {:?}", serialized);

        self.socket.send_with_str(serialized.as_str()).expect("tx to master");
    }

    fn chan(&self) -> &BroadcastChannel {
        &self.chan
    }
}

struct JsRuntimeIo {
    inner: &'static RefCell<JsRuntimeIoImpl>
}

// SAFETY: There is one thread.
unsafe impl Send for JsRuntimeIo {}
unsafe impl Sync for JsRuntimeIo {}

impl RuntimeIo for JsRuntimeIo {
    fn rx(&self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        self.inner.try_borrow_mut().expect("rx inputs").rx()
    }

    fn tx(&self, message: RuntimeMessage) {
        self.inner.try_borrow_mut().expect("tx updates").tx(message)
    }
}

impl JsRuntimeIo {
    async fn new_static(glob: &DedicatedWorkerGlobalScope) -> &'static Self {
        Box::leak(Box::new(Self {
            inner: JsRuntimeIoImpl::new_static(glob).await
        }))
    }

    fn send_to_view(&self, msg: &JsValue) {
        self.inner.borrow().chan().post_message(msg).expect("chan tx send_to_view");
    }
}

#[wasm_bindgen]
pub fn main() { spawn_local(async {
    wasm_logger::init(wasm_logger::Config::new(Level::Info));

    info!("boot client");
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let global_obj = global()
        .dyn_into::<DedicatedWorkerGlobalScope>()
        .unwrap();

    let runtime_io = JsRuntimeIo::new_static(&global_obj).await;
    let runtime = Box::leak(Box::new(RefCell::new(Runtime::new(runtime_io, RuntimeRole::Client))));

    let mut last_t = Date::now();
    let cb = js_fn!(move || {
        let mut rt_lock = runtime.borrow_mut();

        let cur_t = Date::now();
        rt_lock.systems_tick((cur_t - last_t) / 1000.0);
        last_t = cur_t;

        rt_lock.io_tick();

        let ecs = rt_lock.ecs();
        let mut state = Vec::new();
        for eid in ecs.live_eids() {
            let components = ecs.get_entity_anys(eid.clone());

            state.push((eid, components));
        }

        let data = serde_json::to_string(&state).expect("state ser fail");
        runtime_io.send_to_view(&data.into());
    });

    global_obj
        .set_interval_with_callback_and_timeout_and_arguments_0(js_fn_into!(cb), 15)
        .unwrap();

    js_fn_leak!(cb);

    info!("loop inited");
}); }
