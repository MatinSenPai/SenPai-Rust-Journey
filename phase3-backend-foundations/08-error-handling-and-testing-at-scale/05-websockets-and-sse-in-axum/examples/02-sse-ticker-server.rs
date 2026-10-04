//! An endless SSE ticker on 127.0.0.1:3240, for `curl -N`.
//!
//! Each event has an `id`. A client that reconnects sends the last id it saw
//! in the `Last-Event-ID` header, and the ticker carries on from the next one.

use std::convert::Infallible;
use std::time::Duration;

use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::Router;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::{Stream, StreamExt};

async fn ticker(headers: HeaderMap) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let last_seen: u64 = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut id = last_seen;
    let stream = IntervalStream::new(interval(Duration::from_secs(1))).map(move |_| {
        id += 1;
        Ok(Event::default()
            .event("tick")
            .id(id.to_string())
            .data(format!("tick {id}")))
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3240")
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:3240/ticker");
    axum::serve(listener, Router::new().route("/ticker", get(ticker)))
        .await
        .unwrap();
}
