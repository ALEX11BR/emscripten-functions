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

pub enum WebSocketData {
    Text(String),
    RawBuffer(Vec<u8>)
}

pub struct WebSocket {
    id: i32,
    state: WebSocketState,

    open_cb: Option<fn(&mut Self)>,
    error_cb: Option<fn(&mut Self)>,
    close_cb: Option<fn(&mut Self)>,
    message_cb: Option<fn(&mut Self, WebSocketData)>,
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

unsafe extern "C" fn on_close_callback(
    _event_type: ::std::os::raw::c_int,
    _websocket_event: *const EmscriptenWebSocketCloseEvent,
    user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let ws: &mut WebSocket = &mut *(user_data as *mut WebSocket);
    ws.state = WebSocketState::Closed;

    println!("WebSocket CLOSE, CODE: {}, ID : {}", (*_websocket_event).code, ws.id);

    true
}

unsafe extern "C" fn on_message_callback(
    _event_type: ::std::os::raw::c_int,
    websocket_event: *const EmscriptenWebSocketMessageEvent,
    user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let ws: &mut WebSocket = &mut *(user_data as *mut WebSocket);
    println!("WebSocket MESSAGE ID : {}", ws.id);
    if ws.message_cb.is_none() { return true; }

    let fn_cb = ws.message_cb.unwrap();
    if (*websocket_event).isText {
        let tmp_vec = Vec::from_raw_parts((*websocket_event).data, (*websocket_event).numBytes as usize, (*websocket_event).numBytes as usize);
        (fn_cb)(ws, WebSocketData::Text(String::from_utf8(tmp_vec).unwrap()));
    } else {
        let tmp_vec = Vec::from_raw_parts((*websocket_event).data, (*websocket_event).numBytes as usize, (*websocket_event).numBytes as usize);
        (fn_cb)(ws, WebSocketData::RawBuffer(tmp_vec));
    }

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
                close_cb: None,
                message_cb: None
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
            emscripten_websocket_set_onclose_callback_on_thread(
                self.id,
                self as *mut _ as *mut std::os::raw::c_void,
                Some(on_close_callback),
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
            emscripten_websocket_set_onmessage_callback_on_thread(
                self.id,
                self as *mut _ as *mut std::os::raw::c_void,
                Some(on_message_callback),
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
    pub fn set_close_callback(&mut self, cb: Option<fn(&mut Self)>) {
        self.close_cb = cb;
    }
    pub fn set_message_callback(&mut self, cb: Option<fn(&mut Self, WebSocketData)>) {
        self.message_cb = cb;
    }

    pub fn send_utf8_text(&mut self, str: &str) -> bool {
        if self.state == WebSocketState::Opened {
            let text_cstr = CString::new(str).unwrap();
            unsafe {
                emscripten_websocket_send_utf8_text(self.id, text_cstr.as_ptr());
            }

            true
        } else {
            false
        }
    }

    pub fn send_binary(&mut self, data: &mut [u8]) -> bool {
        if self.state == WebSocketState::Opened {
            unsafe {
                emscripten_websocket_send_binary(self.id, data.as_mut_ptr() as *mut std::os::raw::c_void, data.len() as u32);
            }

            true
        } else {
            false
        }
    }
}
