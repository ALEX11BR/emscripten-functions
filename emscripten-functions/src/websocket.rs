use emscripten_functions_sys::websocket::*;
use std::ffi::{CStr, CString};

// NOTE: Need to add EMCC_CFLAGS="[...] -lwebsocket.js" before cargo build --target=wasm32-unknown-emscripten to prevent javascript linking error
pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 0x2 as *mut __pthread;

#[derive(Clone, Copy, PartialEq)]
pub enum WebSocketState {
    WaitingToOpen,
    Opened,
    Closed,
    Error,
}

pub struct WebSocket {
    id: i32,
    state: WebSocketState,

    open_cb: Option<fn(&mut Self)>,
    error_cb: Option<fn(&mut Self)>,
}

unsafe extern "C" fn on_open_callback(
    _event_type: ::std::os::raw::c_int,
    _websocket_event: *const EmscriptenWebSocketOpenEvent,
    user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let ws: &mut WebSocket = &mut *(user_data as *mut WebSocket);
    ws.state = WebSocketState::Opened;

    println!("WebSocket OPEN, ID : {}", ws.id);
    if let Some(fn_cb) = ws.open_cb {
        (fn_cb)(ws);
    } 

    true
}

unsafe extern "C" fn on_error_callback(
    _event_type: ::std::os::raw::c_int,
    _websocket_event: *const EmscriptenWebSocketErrorEvent,
    user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let ws: &mut WebSocket = &mut *(user_data as *mut WebSocket);
    ws.state = WebSocketState::Error;

    println!("WebSocket ERROR, ID : {}", ws.id);

    true
}

impl WebSocket {
    pub fn new() -> Option<WebSocket> {
        if unsafe { emscripten_websocket_is_supported() } {
            Some(WebSocket {
                id: 0,
                state: WebSocketState::Closed,
                open_cb: None,
                error_cb: None,
            })
        } else {
            None
        }
    }

    pub fn connect(&mut self, url: &str) -> bool {
        if self.state != WebSocketState::Closed {
            return false;
        }

        let url_cstr = CString::new(url).unwrap();
        let mut create_attr = EmscriptenWebSocketCreateAttributes {
            url: url_cstr.as_ptr(),
            protocols: std::ptr::null(),
            createOnMainThread: true,
        };

        let socket: i32 = unsafe { emscripten_websocket_new(&mut create_attr) };
        if socket > 0 {
            self.id = socket;
            self.state = WebSocketState::WaitingToOpen;
            self.init_internal_callback();

            true
        } else {
            self.id = 0;
            self.state = WebSocketState::Closed;

            false
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_state(&self) -> WebSocketState {
        self.state
    }

    fn init_internal_callback(&mut self) {
        unsafe {
            emscripten_websocket_set_onopen_callback_on_thread(
                self.id,
                self as *mut _ as *mut std::os::raw::c_void,
                Some(on_open_callback),
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
            emscripten_websocket_set_onerror_callback_on_thread(
                self.id,
                self as *mut _ as *mut std::os::raw::c_void,
                Some(on_error_callback),
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
        }
    }

    pub fn set_open_callback(&mut self, cb: Option<fn(&mut Self)>) {
        self.open_cb = cb;
    }
    pub fn set_error_callback(&mut self, cb: Option<fn(&mut Self)>) {
        self.error_cb = cb;
    }
}
