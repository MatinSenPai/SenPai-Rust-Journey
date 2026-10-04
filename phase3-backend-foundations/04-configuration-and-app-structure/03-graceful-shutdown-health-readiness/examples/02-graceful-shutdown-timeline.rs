//! Shuts a real server down while a request is still running, and prints
//! what each side sees, in order. The shutdown future is a `oneshot` channel
//! this program fires itself, so no keyboard is needed.
//!
//!     cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 02-graceful-shutdown-timeline

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};

async fn get_once(addr: std::net::SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!("GET {path} HTTP/1.1\r\nHost: demo\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).await.unwrap();
    let status = reply.lines().next().unwrap_or("");
    let body = reply.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    format!("{status} / body {body:?}")
}

#[tokio::main]
async fn main() {
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let (started_h, release_h) = (started.clone(), release.clone());
    let app = Router::new().route(
        "/slow",
        get(move || async move {
            started_h.notify_one();
            release_h.notified().await;
            "slow done"
        }),
    );

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

    let slow = tokio::spawn(async move { get_once(addr, "/slow").await });
    started.notified().await;
    println!("1. the slow request is inside its handler");

    stop.send(()).unwrap();
    println!("2. shutdown triggered");

    // The listener is dropped as soon as the signal is seen: poll until a
    // connect fails rather than guessing how long that takes.
    while TcpStream::connect(addr).await.is_ok() {
        tokio::task::yield_now().await;
    }
    println!("3. a new connection is refused");
    println!("4. server finished yet? {}", server.is_finished());

    release.notify_one();
    println!("5. slow request got: {}", slow.await.unwrap());
    println!("6. server returned: {:?}", server.await.unwrap());
}
