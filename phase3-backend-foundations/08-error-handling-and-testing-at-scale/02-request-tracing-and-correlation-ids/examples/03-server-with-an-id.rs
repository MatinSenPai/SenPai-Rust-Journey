//! A real server on 127.0.0.1:3230 with a hand-written request-id middleware:
//! accept a sane `x-request-id`, replace a bad one, put the id on the span and
//! on the response. It does not touch error bodies yet; that is the Build
//! exercise.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 03-server-with-an-id`
//! Then: `curl -i -H 'x-request-id: abc-123' http://127.0.0.1:3230/anime/1`

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use p3_08_02_request_tracing_and_correlation_ids::{new_request_id, routes, RequestId};
use tracing::Instrument;

fn acceptable(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

async fn with_request_id(mut request: Request, next: Next) -> Response {
    let incoming = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok());
    let id = match incoming {
        Some(id) if acceptable(id) => id.to_string(),
        Some(_) => {
            tracing::warn!("ignoring a malformed x-request-id");
            new_request_id()
        }
        None => new_request_id(),
    };
    request.extensions_mut().insert(RequestId(id.clone()));
    let span = tracing::info_span!("request", request_id = %id, method = %request.method(), path = %request.uri().path());
    let mut response = next.run(request).instrument(span).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&id).unwrap());
    response
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();
    let app = routes().layer(from_fn(with_request_id));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3230")
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:3230");
    axum::serve(listener, app).await.unwrap();
}
