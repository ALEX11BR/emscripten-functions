use emscripten_functions_sys::websocket::{
    pthread_t, EmscriptenWebSocketCloseEvent, EmscriptenWebSocketErrorEvent,
    EmscriptenWebSocketMessageEvent, EmscriptenWebSocketOpenEvent, __pthread,
    emscripten_websocket_close, emscripten_websocket_delete,
    emscripten_websocket_get_buffered_amount, emscripten_websocket_get_protocol,
    emscripten_websocket_get_protocol_length, emscripten_websocket_get_url,
    emscripten_websocket_get_url_length, emscripten_websocket_is_supported,
    emscripten_websocket_new, emscripten_websocket_send_binary,
    emscripten_websocket_send_utf8_text, emscripten_websocket_set_onclose_callback_on_thread,
    emscripten_websocket_set_onerror_callback_on_thread,
    emscripten_websocket_set_onmessage_callback_on_thread,
    emscripten_websocket_set_onopen_callback_on_thread, EmscriptenWebSocketCreateAttributes,
    EMSCRIPTEN_RESULT_SUCCESS,
};
use std::ffi::CString;

pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 0x2 as *mut __pthread;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    Connecting,
    Opened,
    Closing,
    Closed,
    Error,
}

pub enum WebSocketData {
    Text(String),
    RawBuffer(Vec<u8>),
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
    ws.clear_internal_callback();

    if let Some(fn_cb) = ws.error_cb {
        (fn_cb)(ws);
    }

    true
}

unsafe extern "C" fn on_close_callback(
    _event_type: ::std::os::raw::c_int,
    _websocket_event: *const EmscriptenWebSocketCloseEvent,
    user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let ws: &mut WebSocket = &mut *(user_data as *mut WebSocket);
    ws.state = WebSocketState::Closed;
    ws.clear_internal_callback();

    if let Some(fn_cb) = ws.close_cb {
        (fn_cb)(ws);
    }

    true
}

unsafe extern "C" fn on_message_callback(
    _event_type: ::std::os::raw::c_int,
    websocket_event: *const EmscriptenWebSocketMessageEvent,
    user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let ws: &mut WebSocket = &mut *(user_data as *mut WebSocket);
    if ws.message_cb.is_none() {
        return true;
    }

    let fn_cb = ws.message_cb.unwrap();
    if (*websocket_event).isText {
        let tmp_vec = Vec::from_raw_parts(
            (*websocket_event).data,
            (*websocket_event).numBytes as usize,
            (*websocket_event).numBytes as usize,
        );
        (fn_cb)(ws, WebSocketData::Text(String::from_utf8(tmp_vec).unwrap()));
    } else {
        let tmp_vec = Vec::from_raw_parts(
            (*websocket_event).data,
            (*websocket_event).numBytes as usize,
            (*websocket_event).numBytes as usize,
        );
        (fn_cb)(ws, WebSocketData::RawBuffer(tmp_vec));
    }

    true
}

impl WebSocket {
    /// Create a new WebSocket handler object. One object should be created by socket.
    ///
    /// # Examples
    /// ```rust
    /// let mut ws = WebSocket::new().unwrap();
    /// ws.connect("wss://echo.websocket.org/");
    /// ```
    pub fn new() -> Option<WebSocket> {
        if unsafe { emscripten_websocket_is_supported() } {
            Some(WebSocket {
                id: 0,
                state: WebSocketState::Closed,
                open_cb: None,
                error_cb: None,
                close_cb: None,
                message_cb: None,
            })
        } else {
            None
        }
    }

    /// The connect method open a socket connection to the provided URL. This method should only be
    /// call if the current state of the handler is Closed, else the function will do nothing and
    /// return false.
    ///
    /// # Examples
    /// ```rust
    /// let mut ws = WebSocket::new().unwrap();
    /// ws.connect("wss://echo.websocket.org/");
    /// ```
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
            self.state = WebSocketState::Connecting;
            self.init_internal_callback();

            true
        } else {
            self.id = 0;
            self.state = WebSocketState::Closed;

            false
        }
    }

    /// Get id field of websocket.
    pub fn get_id(&self) -> i32 {
        self.id
    }
    /// Get current websocket state.
    pub fn get_state(&self) -> WebSocketState {
        self.state
    }
    /// Get the WebSocket.bufferedAmount field into bufferedAmount.    
    pub fn get_buffered_amount(&self) -> usize {
        let mut size: usize = 0;
        unsafe {
            emscripten_websocket_get_buffered_amount(self.id, &mut size);
        }

        size
    }
    /// Get the websocket URL field.    
    pub fn get_url(&self) -> String {
        let mut size: i32 = 0;
        unsafe {
            emscripten_websocket_get_url_length(self.id, &mut size);
            let mut url_raw = vec![0u8; size as usize];
            let url_raw_ptr = url_raw.as_mut_ptr() as *mut i8;
            emscripten_websocket_get_url(self.id, url_raw_ptr, 26);
            String::from_utf8(url_raw).unwrap()
        }
    }
    /// Get the websocket protocol field.    
    pub fn get_protocol(&self) -> String {
        let mut size: i32 = 0;
        unsafe {
            emscripten_websocket_get_protocol_length(self.id, &mut size);
            let mut protocol_raw = vec![0u8; size as usize];
            let protocol_raw_ptr = protocol_raw.as_mut_ptr() as *mut i8;
            emscripten_websocket_get_protocol(self.id, protocol_raw_ptr, 26);
            String::from_utf8(protocol_raw).unwrap()
        }
    }

    /// Set all internal callbacks for current object.    
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
    /// Clear all internal callbacks for current object.    
    pub fn clear_internal_callback(&mut self) {
        unsafe {
            emscripten_websocket_set_onopen_callback_on_thread(
                self.id,
                std::ptr::null_mut::<std::os::raw::c_void>(),
                None,
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
            emscripten_websocket_set_onerror_callback_on_thread(
                self.id,
                std::ptr::null_mut::<std::os::raw::c_void>(),
                None,
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
            emscripten_websocket_set_onclose_callback_on_thread(
                self.id,
                std::ptr::null_mut::<std::os::raw::c_void>(),
                None,
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
            emscripten_websocket_set_onmessage_callback_on_thread(
                self.id,
                std::ptr::null_mut::<std::os::raw::c_void>(),
                None,
                EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD,
            );
        }
    }

    /// Set on open callback for current object.    
    pub fn set_open_callback(&mut self, cb: Option<fn(&mut Self)>) {
        self.open_cb = cb;
    }
    /// Set on error callback for current object.    
    pub fn set_error_callback(&mut self, cb: Option<fn(&mut Self)>) {
        self.error_cb = cb;
    }
    /// Set on close callback for current object.    
    pub fn set_close_callback(&mut self, cb: Option<fn(&mut Self)>) {
        self.close_cb = cb;
    }
    /// Set on message callback for current object.    
    pub fn set_message_callback(&mut self, cb: Option<fn(&mut Self, WebSocketData)>) {
        self.message_cb = cb;
    }

    /// Send UTF-8 formatted string through websocket. The state of current socket should be
    /// opened else the function will return false.    
    ///
    /// # Examples
    /// ```rust
    /// let mut ws = WebSocket::new().unwrap();
    /// ws.connect("wss://echo.websocket.org/");
    /// ws.send_utf8_text("foo");
    /// ```
    pub fn send_utf8_text(&mut self, str: &str) -> bool {
        let text_cstr = CString::new(str).unwrap();
        unsafe {
            let result = emscripten_websocket_send_utf8_text(self.id, text_cstr.as_ptr());
            (result as u32) == EMSCRIPTEN_RESULT_SUCCESS
        }
    }

    /// Send UTF-8 raw data through websocket. The state of current socket should be
    /// opened else the function will return false.
    ///
    /// # Examples
    /// ```rust
    /// let mut ws = WebSocket::new().unwrap();
    /// ws.connect("wss://echo.websocket.org/");
    /// ws.send_binary([42, 69]);
    /// ```
    pub fn send_binary(&mut self, data: &mut [u8]) -> bool {
        unsafe {
            let result = emscripten_websocket_send_binary(
                self.id,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                data.len() as u32,
            );

            (result as u32) == EMSCRIPTEN_RESULT_SUCCESS
        }
    }

    /// Close the current websocket. This function will clear all
    /// callbacks and trigger on close callback.
    pub fn close(&mut self, code: u16, reason: &str) {
        self.state = WebSocketState::Closing;
        let reason_cstr = CString::new(reason).unwrap();
        unsafe {
            emscripten_websocket_close(self.id, code, reason_cstr.as_ptr());
        }
    }
}

impl Drop for WebSocket {
    fn drop(&mut self) {
        unsafe {
            emscripten_websocket_delete(self.id);
        }
    }
}
