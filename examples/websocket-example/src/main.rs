use emscripten_functions::websocket::{WebSocket, WebSocketData};

fn main() {
    let mut ws = WebSocket::new().unwrap();
    ws.set_open_callback(Some(|ws| {
        // Send works after the WebSocket is fully open.
        ws.send_utf8_text("Hello");
    }));
    ws.set_message_callback(Some(|_ws, wsd| match wsd {
        WebSocketData::Text(text) => {
            println!("Received string: {}", text);
        }
        WebSocketData::RawBuffer(buf) => {
            println!("Received bytes: {:?}", buf);
        }
    }));
    ws.connect("wss://echo.websocket.org/");
    ws.send_utf8_text("");
    // For whatever reasons, a (dummy) send has to happen for the WebSocket to be fully open.
    // It will not send anything to the server and will generate an error in the console.
}
