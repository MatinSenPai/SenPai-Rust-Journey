//! Open a WebSocket to an `axum` echo route with nothing but a `TcpStream`:
//! the HTTP 101 handshake, then one masked frame out and one plain frame back.

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn ws_echo(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|mut socket: WebSocket| async move {
        while let Some(Ok(msg)) = socket.recv().await {
            if socket.send(msg).await.is_err() {
                break;
            }
        }
    })
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, Router::new().route("/ws", get(ws_echo)))
            .await
            .unwrap();
    });

    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = "GET /ws HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
                   Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    // The Date header changes on every run, so it is not printed.
    let mut reader = BufReader::new(stream);
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        if !line.starts_with("date:") {
            print!("< {line}");
        }
        if line == "\r\n" {
            break;
        }
    }

    // A client frame: FIN + text opcode, MASK bit + length 5, 4-byte mask, masked payload.
    let mask = [0x01, 0x02, 0x03, 0x04];
    let mut frame = vec![0x81, 0x80 | 5];
    frame.extend_from_slice(&mask);
    frame.extend(b"hello".iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
    reader.get_mut().write_all(&frame).await.unwrap();
    println!("> {:02x?}", frame);

    let mut reply = [0u8; 7];
    reader.read_exact(&mut reply).await.unwrap();
    println!(
        "< {:02x?}  ({:?})",
        reply,
        std::str::from_utf8(&reply[2..]).unwrap()
    );
    server.abort();
}
