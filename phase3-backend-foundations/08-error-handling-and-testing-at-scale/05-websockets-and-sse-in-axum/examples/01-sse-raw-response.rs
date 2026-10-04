//! Serve three SSE events and read the raw bytes of the response over a plain
//! `TcpStream`: no `Content-Length`, `Transfer-Encoding: chunked`, one chunk
//! per event.

use std::convert::Infallible;

use axum::response::sse::{Event, Sse};
use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn ticks() -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let events = (1..=3).map(|n| Ok(Event::default().event("tick").data(format!("tick {n}"))));
    Sse::new(tokio_stream::iter(events))
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, Router::new().route("/ticks", get(ticks)))
            .await
            .unwrap();
    });

    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = "GET /ticks HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).await.unwrap();

    // Show every CRLF as visible escape characters. The Date
    // header is left out because it changes on every run.
    for line in raw.split_inclusive("\r\n") {
        if !line.starts_with("date:") {
            print!("{}", line.replace("\r\n", "\\r\\n\n"));
        }
    }
    server.abort();
}
