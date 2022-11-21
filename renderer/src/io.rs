use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, BroadcastChannel};

use common::{
    js_fn_into, js_fn, js_fn_leak, message_event_to_runtime_message, message_event_to
};
use common::runtime::{RuntimeMessage, RuntimeIo};
use common::input::Input;

struct RendererRuntimeIoImpl {
    chan: BroadcastChannel,
    rx_queue: Option<Vec<RuntimeMessage>>
}

impl RendererRuntimeIoImpl {
    fn new_static() -> &'static RefCell<Self> {
        let instance = Box::leak(Box::new(RefCell::new(Self {
            chan: BroadcastChannel::new("woods-renderer").expect("chan open fail"),
            rx_queue: Some(Vec::new())
        })));

        let handle_client_message = js_fn!(|event: MessageEvent| {
            let message = message_event_to_runtime_message!(event);

            instance
                .try_borrow_mut().expect("on handle_client_message")
                .rx_queue
                .as_mut().expect("rx_queue unset")
                .push(message);
        });

        instance
            .borrow_mut()
            .chan
            .add_event_listener_with_callback("message".into(), js_fn_into!(handle_client_message))
            .unwrap();

        js_fn_leak!(handle_client_message);

        instance
    }

    fn rx(&mut self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        let rx_queue = self.rx_queue.take().expect("rx_queue unset");

        self.rx_queue = Some(Vec::new());

        (Vec::new(), rx_queue)
    }
}

pub struct RendererRuntimeIo {
    inner: &'static RefCell<RendererRuntimeIoImpl>
}

// SAFETY: There is one thread.
unsafe impl Send for RendererRuntimeIo {}
unsafe impl Sync for RendererRuntimeIo {}

impl RuntimeIo for RendererRuntimeIo {
    fn rx(&self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        self.inner.try_borrow_mut().expect("on rx").rx()
    }

    fn tx(&self, _: RuntimeMessage, _: bool) {
        unreachable!("renderer shouldn't tx");
    }
}

impl RendererRuntimeIo {
    pub fn new_static() -> &'static Self {
        Box::leak(Box::new(Self {
            inner: RendererRuntimeIoImpl::new_static()
        }))
    }
}
