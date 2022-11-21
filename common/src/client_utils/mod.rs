#[macro_export]
macro_rules! init_console_logging {
    () => {
        ::wasm_logger::init(::wasm_logger::Config::new(Level::Info));
        ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}

#[macro_export]
macro_rules! message_event_to {
    ($e: ident, $t: ty) => {
        {
            let data_string = $e
                .data()
                .dyn_into::<js_sys::JsString>().expect("message event payload not a string")
                .as_string().unwrap();

            serde_json::from_str::<$t>(data_string.as_str())
                .expect("message event deserialization failed")
        }
    }
}

#[macro_export]
macro_rules! message_event_to_runtime_message {
    ($e: ident) => {
        {
            let data_string = $e
                .data()
                .dyn_into::<js_sys::JsString>().expect("message event payload not a string")
                .as_string().unwrap();

            let message = message_event_to!($e, $crate::runtime::RuntimeMessage);

            log::debug!("rx message {:?}", message);
            match &message {
                RuntimeMessage::ComponentUpdate(..) => {},
                other => log::info!("rx major message {:?}", other)
            };

            message
        }
    }
}

#[macro_export]
macro_rules! block_pattern {
    (|$r: ident| $($b: tt)*) => {
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            (|$r| $($b)*)(resolve);
        }))
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! js_fn_into {
    ($s: ident) => {
        $s.as_ref().unchecked_ref()
    };
}

#[macro_export]
macro_rules! js_fn_leak {
    ($s: ident) => {
        $s.forget()
    }
}

#[macro_export]
macro_rules! global_scope {
    ($t: ty) => {
        js_sys::global()
            .dyn_into::<$t>()
            .expect("incorrect global object");
    }
}
