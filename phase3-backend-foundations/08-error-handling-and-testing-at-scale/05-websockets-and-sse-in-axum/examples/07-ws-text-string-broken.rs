//! DELIBERATELY BROKEN — expected: E0308
//!
//! In axum 0.8 `Message::Text` holds a `Utf8Bytes`, not a `String`.

use axum::extract::ws::{Message, WebSocket};

async fn greet(mut socket: WebSocket) {
    let hello = String::from("hello");
    socket.send(Message::Text(hello)).await.unwrap();
}

fn main() {
    // Only the type matters here; nothing connects.
    let _ = greet;
}
