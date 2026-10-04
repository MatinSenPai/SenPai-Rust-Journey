//! Two handlers, one `Router`, four requests.
//!
//! Every request goes through the router with `oneshot` — no port, no socket.
//! "Testing without a socket: `oneshot`" in the README explains how `send`
//! below does that.
//!
//!     cargo run -p p3-02-01-routing-handlers-extractors --example 01-first-router

use axum::body::{to_bytes, Body};
use axum::extract::Path;
use axum::http::{header, Request};
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

async fn hello() -> &'static str {
    "Hello, world!"
}

async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}

async fn send(app: &Router, method: &str, uri: &str) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let allow = response.headers().get(header::ALLOW).cloned();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&bytes);
    print!("{method:<4} {uri:<14} -> {status}  body: {body:?}");
    match allow {
        Some(allow) => println!("  allow: {allow:?}"),
        None => println!(),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet));

    send(&app, "GET", "/").await;
    send(&app, "GET", "/greet/senpai").await;
    send(&app, "GET", "/nope").await;
    send(&app, "POST", "/greet/senpai").await;
}
