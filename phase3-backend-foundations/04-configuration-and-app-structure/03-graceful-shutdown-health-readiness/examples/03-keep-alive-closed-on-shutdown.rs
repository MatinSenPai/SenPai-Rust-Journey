//! 3.1.3 said an HTTP/1.1 connection stays open between requests. This
//! program holds one open, shuts the server down, and shows what happens to
//! the idle connection.
//!
//!     cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 03-keep-alive-closed-on-shutdown

use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

/// Sends `GET /health` on an open connection and reads until the body `ok`.
async fn health(stream: &mut TcpStream) -> String {
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: demo\r\n\r\n")
        .await
        .unwrap();
    let mut reply = String::new();
    let mut buf = [0u8; 256];
    while !reply.ends_with("ok") {
        let n = stream.read(&mut buf).await.unwrap();
        reply.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    reply.lines().next().unwrap().to_string()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(|| async { "ok" }));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                stopped.await.ok();
            })
            .await
    });

    let mut stream = TcpStream::connect(addr).await.unwrap();
    println!("request 1 on the connection: {}", health(&mut stream).await);
    println!("request 2, same connection:  {}", health(&mut stream).await);

    stop.send(()).unwrap();
    let mut buf = [0u8; 256];
    let n = stream.read(&mut buf).await.unwrap();
    println!("after shutdown, read() returned {n} bytes (end of stream)");
    println!("server returned: {:?}", server.await.unwrap());
}
