//! Test helpers: a real server on `127.0.0.1:0`, a raw-TCP SSE client that
//! decodes chunked encoding, and a hand-written WebSocket client.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::timeout;

/// Serves `app` on an ephemeral port. Abort the handle to stop the server.
pub async fn spawn(app: Router) -> (SocketAddr, JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, handle)
}

/// Reads the status line and headers, returning them as one string
/// (the header block without the final blank line).
pub async fn read_head(reader: &mut BufReader<TcpStream>) -> String {
    let mut head = String::new();
    loop {
        let mut line = String::new();
        let n = timeout(Duration::from_secs(5), reader.read_line(&mut line))
            .await
            .expect("timed out reading the response head")
            .unwrap();
        assert!(n > 0, "connection closed before the head ended");
        if line == "\r\n" {
            return head;
        }
        head.push_str(&line);
    }
}

/// Opens `GET path` as an SSE client and returns the reader and the head.
pub async fn sse_connect(addr: SocketAddr, path: &str) -> (BufReader<TcpStream>, String) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nAccept: text/event-stream\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut reader = BufReader::new(stream);
    let head = read_head(&mut reader).await;
    (reader, head)
}

/// Reads one HTTP/1.1 chunk and returns its data as text.
pub async fn next_chunk(reader: &mut BufReader<TcpStream>) -> String {
    let mut size_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut size_line))
        .await
        .expect("timed out waiting for the next chunk")
        .unwrap();
    let size = usize::from_str_radix(size_line.trim(), 16).expect("chunk size line");
    let mut data = vec![0u8; size + 2]; // the data and its trailing \r\n
    reader.read_exact(&mut data).await.unwrap();
    data.truncate(size);
    String::from_utf8(data).unwrap()
}

/// `POST path` with a text body on a fresh connection; returns (status line, body).
pub async fn post(addr: SocketAddr, path: &str, body: &str) -> (String, String) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).await.unwrap();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap();
    (head.lines().next().unwrap().to_string(), body.to_string())
}

// ---------------------------------------------------------------- WebSocket

/// The example key from RFC 6455 section 1.3. Its accept value is known, so
/// the test can check the server's handshake without any hashing code.
pub const WS_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
pub const WS_ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

/// Performs the opening handshake and returns the reader and the 101 head.
pub async fn ws_connect(addr: SocketAddr, path: &str) -> (BufReader<TcpStream>, String) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Key: {WS_KEY}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut reader = BufReader::new(stream);
    let head = read_head(&mut reader).await;
    (reader, head)
}

/// Sends one masked frame (clients must mask). Payload under 126 bytes.
pub async fn ws_send(reader: &mut BufReader<TcpStream>, opcode: u8, payload: &[u8]) {
    assert!(payload.len() < 126);
    let mask = [0x12, 0x34, 0x56, 0x78];
    let mut frame = vec![0x80 | opcode, 0x80 | payload.len() as u8];
    frame.extend_from_slice(&mask);
    frame.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
    reader.get_mut().write_all(&frame).await.unwrap();
}

/// Reads one server frame (servers do not mask) and returns (opcode, payload).
pub async fn ws_recv(reader: &mut BufReader<TcpStream>) -> (u8, Vec<u8>) {
    let mut head = [0u8; 2];
    timeout(Duration::from_secs(5), reader.read_exact(&mut head))
        .await
        .expect("timed out waiting for a frame")
        .unwrap();
    assert_eq!(head[1] & 0x80, 0, "a server frame must not be masked");
    let len = (head[1] & 0x7f) as usize;
    assert!(len < 126);
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await.unwrap();
    (head[0] & 0x0f, payload)
}
