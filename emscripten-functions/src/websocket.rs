use emscripten_functions_sys::websocket::*;
use std::ffi::{CStr, CString};

// NOTE: Need to add EMCC_CFLAGS="[...] -lwebsocket.js" before cargo build --target=wasm32-unknown-emscripten to prevent javascript linking error

pub struct WebSocket {
    id: i32,
}
impl WebSocket {
    pub fn new(url: &str) -> Option<WebSocket> {
        let url_cstr  = CString::new(url).unwrap();
        if unsafe { emscripten_websocket_is_supported() } {
            let mut create_attr = EmscriptenWebSocketCreateAttributes {
                url: url_cstr.as_ptr(),
                protocols: std::ptr::null(),
                createOnMainThread: true,
            };

            let socket: i32 = unsafe { emscripten_websocket_new(&mut create_attr) };
            Some(WebSocket { id: socket })
        } else {
            None
        }
    }
}
