mod common;
use common::*;
use p3_08_05_websockets_and_sse_in_axum::{app, Hub};

#[tokio::test]
async fn handshake_answers_101_with_the_right_accept_value() {
    let (addr, server) = spawn(app(Hub::new(4))).await;
    let (_reader, head) = ws_connect(addr, "/ws").await;
    assert!(head.starts_with("HTTP/1.1 101"), "{head}");
    assert!(
        head.to_ascii_lowercase().contains(&format!(
            "sec-websocket-accept: {}",
            WS_ACCEPT.to_ascii_lowercase()
        )),
        "{head}"
    );
    server.abort();
}

#[tokio::test]
async fn text_commands_round_trip() {
    let (addr, server) = spawn(app(Hub::new(4))).await;
    let (mut ws, _) = ws_connect(addr, "/ws").await;

    ws_send(&mut ws, 0x1, b"ping").await;
    assert_eq!(ws_recv(&mut ws).await, (0x1, b"pong".to_vec()));

    ws_send(&mut ws, 0x1, "echo こん".as_bytes()).await;
    assert_eq!(ws_recv(&mut ws).await, (0x1, "こん".as_bytes().to_vec()));

    ws_send(&mut ws, 0x1, b"dance").await;
    assert_eq!(ws_recv(&mut ws).await, (0x1, b"unknown: dance".to_vec()));
    server.abort();
}

#[tokio::test]
async fn a_close_frame_is_answered_with_close() {
    let (addr, server) = spawn(app(Hub::new(4))).await;
    let (mut ws, _) = ws_connect(addr, "/ws").await;
    ws_send(&mut ws, 0x8, &[0x03, 0xe8]).await; // status 1000, normal closure
    let (opcode, _) = ws_recv(&mut ws).await;
    assert_eq!(opcode, 0x8);
    server.abort();
}
